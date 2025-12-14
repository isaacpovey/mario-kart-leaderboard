use crate::config::Config;
use crate::db::DbPool;
use crate::graphql::{GraphQLContext, Schema};
use crate::services::notification_manager::NotificationManager;
use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    extract::Extension as AxumExtension,
    http::{HeaderMap, StatusCode, header::ACCEPT},
    response::{
        sse::{Event, KeepAlive, Sse},
        Html, IntoResponse, Response,
    },
};
use futures::stream::StreamExt;
use std::convert::Infallible;
use std::time::Duration;
use uuid::Uuid;

/// HTTP GraphQL handler for queries and mutations
pub async fn graphql_handler(
    schema: AxumExtension<Schema>,
    pool: AxumExtension<DbPool>,
    config: AxumExtension<Config>,
    notification_manager: AxumExtension<NotificationManager>,
    group_id: AxumExtension<Option<Uuid>>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    let ctx = GraphQLContext::new(pool.0, *group_id, notification_manager.0);

    schema
        .0
        .execute(req.into_inner().data(ctx).data(config.0))
        .await
        .into()
}

/// SSE GraphQL handler for subscriptions
///
/// Handles GraphQL subscriptions over Server-Sent Events (SSE)
/// Compatible with urql's @urql/exchange-sse in distinct connections mode
pub async fn graphql_sse_handler(
    schema: AxumExtension<Schema>,
    pool: AxumExtension<DbPool>,
    config: AxumExtension<Config>,
    notification_manager: AxumExtension<NotificationManager>,
    group_id: AxumExtension<Option<Uuid>>,
    req: GraphQLRequest,
) -> Response {
    // Validate authentication for subscriptions
    if group_id.is_none() {
        return (
            StatusCode::UNAUTHORIZED,
            "Authentication required for subscriptions",
        )
            .into_response();
    }

    let ctx = GraphQLContext::new(pool.0, *group_id, notification_manager.0);

    // Clone the Arc-wrapped values for use in the stream
    let schema_clone = schema.0.clone();
    let config_clone = config.0.clone();

    // Execute the subscription
    let request = req.into_inner().data(ctx).data(config_clone);

    // Convert GraphQL response stream to SSE events
    let sse_stream = async_stream::stream! {
        let mut stream = schema_clone.execute_stream(request);

        while let Some(response) = stream.next().await {
            let json = match serde_json::to_string(&response) {
                Ok(json) => json,
                Err(e) => {
                    tracing::error!("Failed to serialize GraphQL response: {}", e);
                    continue;
                }
            };

            yield Ok::<_, Infallible>(Event::default().event("next").data(json));
        }

        // Send complete event when stream ends
        yield Ok(Event::default().event("complete"));
    };

    Sse::new(sse_stream)
        .keep_alive(
            KeepAlive::new()
                .interval(Duration::from_secs(15))
                .text("keep-alive"),
        )
        .into_response()
}

/// Unified GraphQL handler that routes based on Accept header
///
/// - If Accept header contains "text/event-stream": handles as SSE subscription
/// - Otherwise: handles as regular query/mutation with JSON response
pub async fn unified_graphql_handler(
    schema: AxumExtension<Schema>,
    pool: AxumExtension<DbPool>,
    config: AxumExtension<Config>,
    notification_manager: AxumExtension<NotificationManager>,
    group_id: AxumExtension<Option<Uuid>>,
    headers: HeaderMap,
    req: GraphQLRequest,
) -> Response {
    // Check Accept header for SSE - only treat as SSE if text/event-stream is the primary type
    // (first in the Accept header list), not just if it's one of many acceptable types
    let accepts_sse = headers
        .get(ACCEPT)
        .and_then(|v| v.to_str().ok())
        .map(|v| {
            v.split(',')
                .next()
                .map(|first| first.trim().starts_with("text/event-stream"))
                .unwrap_or(false)
        })
        .unwrap_or(false);

    if accepts_sse {
        // Handle as SSE subscription
        tracing::info!("NOTIFY STEP 4: SSE mode detected, setting up subscription stream");

        if group_id.is_none() {
            tracing::warn!("NOTIFY STEP 4: Unauthorized - no group_id");
            return (
                StatusCode::UNAUTHORIZED,
                "Authentication required for subscriptions",
            )
                .into_response();
        }

        tracing::info!("NOTIFY STEP 4: Authenticated with group_id={:?}", group_id);

        let ctx = GraphQLContext::new(pool.0, *group_id, notification_manager.0);
        let schema_clone = schema.0.clone();
        let config_clone = config.0.clone();
        let request = req.into_inner().data(ctx).data(config_clone);

        tracing::info!("NOTIFY STEP 4: Executing GraphQL subscription stream");

        let sse_stream = async_stream::stream! {
            let mut stream = schema_clone.execute_stream(request);

            tracing::info!("NOTIFY STEP 4: Stream created, waiting for items...");

            while let Some(response) = stream.next().await {
                tracing::info!("NOTIFY STEP 4: Received item from GraphQL stream");

                let json = match serde_json::to_string(&response) {
                    Ok(json) => json,
                    Err(e) => {
                        tracing::error!("Failed to serialize GraphQL response: {}", e);
                        continue;
                    }
                };

                tracing::info!("NOTIFY STEP 4: Sending SSE event with data: {}",
                    if json.len() > 200 { &json[..200] } else { &json }
                );

                yield Ok::<_, Infallible>(Event::default().event("next").data(json));
            }

            tracing::info!("NOTIFY STEP 4: Stream ended, sending complete event");
            yield Ok(Event::default().event("complete"));
        };

        Sse::new(sse_stream)
            .keep_alive(
                KeepAlive::new()
                    .interval(Duration::from_secs(15))
                    .text("keep-alive"),
            )
            .into_response()
    } else {
        // Handle as regular query/mutation
        let ctx = GraphQLContext::new(pool.0, *group_id, notification_manager.0);

        let response: GraphQLResponse = schema
            .0
            .execute(req.into_inner().data(ctx).data(config.0))
            .await
            .into();

        response.into_response()
    }
}

pub async fn graphql_playground() -> impl IntoResponse {
    Html(playground_source(GraphQLPlaygroundConfig::new("/graphql")))
}
