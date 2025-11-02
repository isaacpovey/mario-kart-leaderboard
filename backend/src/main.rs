use axum::{
    middleware,
    routing::{get, post},
    Extension, Router,
};
use mario_kart_leaderboard_backend::{
    config::Config,
    db::create_pool,
    graphql::build_schema,
    handlers::{graphql_handler, graphql_playground},
    middleware::auth::auth_middleware,
};
use tower_http::cors::{AllowOrigin, CorsLayer};

#[shuttle_runtime::main]
async fn main(
    #[shuttle_runtime::Secrets] secrets: shuttle_runtime::SecretStore
) -> shuttle_axum::ShuttleAxum {
    let config = Config::from_env(secrets).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
    let pool = create_pool(&config.database_url).await.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

    eprintln!("Connected to database");

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
        .layer(cors);

    Ok(app.into())
}
