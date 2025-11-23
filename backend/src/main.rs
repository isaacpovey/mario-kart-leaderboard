use axum::{
    Extension, Router, middleware,
    routing::{get, post},
};
use mario_kart_leaderboard_backend::error::AppError;
use mario_kart_leaderboard_backend::{
    config::Config,
    db::create_pool,
    graphql::build_schema,
    handlers::{graphql_playground, unified_graphql_handler},
    middleware::auth::auth_middleware,
    observability::{init_telemetry, shutdown_telemetry},
    services::notification_manager::NotificationManager,
};
use tower_http::LatencyUnit;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing;

#[tokio::main]
async fn main() -> Result<(), AppError> {
    let config = Config::from_env()?;

    init_telemetry(&config.service_name, config.otlp_endpoint.as_deref())?;

    tracing::info_span!("app_startup").in_scope(|| {
        tracing::info!("Application starting up");
    });

    let pool = create_pool(&config.database_url, config.database_max_connections).await?;

    let schema = build_schema();

    // Initialize notification manager for GraphQL subscriptions
    let notification_manager = NotificationManager::new();

    // Start PostgreSQL LISTEN in background
    notification_manager
        .clone()
        .start_listener(&config.database_url)
        .await?;

    tracing::info!("Notification manager initialized and listening to PostgreSQL");

    // Configure CORS with specific allowed origin
    let allowed_origins: Vec<_> = config
        .cors_origins
        .iter()
        .filter_map(|origin| origin.parse().ok())
        .collect();

    tracing::info!("Configured CORS origins: {:?}", config.cors_origins);

    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::list(allowed_origins))
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::OPTIONS,
        ])
        .allow_headers([
            axum::http::header::CONTENT_TYPE,
            axum::http::header::AUTHORIZATION,
            axum::http::header::ACCEPT,
            axum::http::header::CACHE_CONTROL,
        ])
        .allow_credentials(true);

    let app = Router::new()
        .route("/", get(graphql_playground))
        .route(
            "/graphql",
            post(unified_graphql_handler).get(unified_graphql_handler),
        )
        .layer(middleware::from_fn_with_state(
            config.clone(),
            auth_middleware,
        ))
        .layer(Extension(schema))
        .layer(Extension(pool))
        .layer(Extension(notification_manager))
        .layer(Extension(config.clone()))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(tracing::Level::INFO))
                .on_response(
                    DefaultOnResponse::new()
                        .level(tracing::Level::INFO)
                        .latency_unit(LatencyUnit::Millis),
                ),
        )
        .layer(cors);

    let addr = config.server_addr();
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    tracing::info!("GraphQL server running at http://{}/graphql", addr);
    tracing::info!("GraphQL SSE subscriptions: GET http://{}/graphql", addr);
    tracing::info!("GraphQL Playground available at http://{}/", addr);

    axum::serve(listener, app).await?;

    shutdown_telemetry();

    Ok(())
}
