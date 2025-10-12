use crate::config::Config;
use crate::graphql::{GraphQLContext, Schema};
use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    extract::Extension as AxumExtension,
    response::{Html, IntoResponse},
};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn graphql_handler(
    AxumExtension(schema): AxumExtension<Schema>,
    AxumExtension(pool): AxumExtension<PgPool>,
    AxumExtension(config): AxumExtension<Config>,
    AxumExtension(group_id): AxumExtension<Option<Uuid>>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    let ctx = GraphQLContext::new(pool, group_id);

    schema
        .execute(req.into_inner().data(ctx).data(config))
        .await
        .into()
}

pub async fn graphql_playground() -> impl IntoResponse {
    Html(playground_source(GraphQLPlaygroundConfig::new("/graphql")))
}
