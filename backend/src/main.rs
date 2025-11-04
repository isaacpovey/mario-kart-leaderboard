use axum::{
    Extension, Router, middleware,
    routing::{get, post},
};
use mario_kart_leaderboard_backend::error::AppError;
use mario_kart_leaderboard_backend::{
    config::Config,
    db::create_pool,
    graphql::build_schema,
    handlers::{graphql_handler, graphql_playground},
    middleware::auth::auth_middleware,
};
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), AppError> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                "mario_kart_leaderboard_backend=info,tower_http=info,axum=info".into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Config::from_env()?;
    let pool = create_pool(&config.database_url, config.database_max_connections).await?;

    tracing::info!("Connected to database");

    let schema = build_schema();

    // Configure CORS with specific allowed origin
    let allowed_origins: Vec<_> = config
        .cors_origins
        .iter()
        .filter_map(|origin| origin.parse().ok())
        .collect();

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
        ]);

    let app = Router::new()
        .route("/", get(graphql_playground))
        .route("/graphql", post(graphql_handler).get(graphql_playground))
        .layer(middleware::from_fn_with_state(
            config.clone(),
            auth_middleware,
        ))
        .layer(Extension(schema))
        .layer(Extension(pool))
        .layer(Extension(config.clone()))
        .layer(TraceLayer::new_for_http())
        .layer(cors);

    let addr = config.server_addr();
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    tracing::info!("GraphQL server running at http://{}/graphql", addr);
    tracing::info!("GraphQL Playground available at http://{}/", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
