use mario_kart_leaderboard_backend::{
    config::Config,
    graphql::schema::{Schema, build_schema},
};
use sqlx::{PgPool, postgres::PgPoolOptions};
use testcontainers::ContainerAsync;
use testcontainers::runners::AsyncRunner;
use testcontainers_modules::postgres::Postgres;

pub struct TestContext {
    pub pool: PgPool,
    pub schema: Schema,
    pub config: Config,
    pub _container: ContainerAsync<Postgres>,
}

/// Setup a test database with migrations applied
pub async fn setup_test_db() -> TestContext {
    // Testcontainers configuration is handled via ~/.testcontainers.properties
    // which sets docker.host and ryuk.disabled for Podman compatibility

    // Start PostgreSQL container
    let postgres_image = Postgres::default();
    let container = postgres_image
        .start()
        .await
        .expect("Failed to start postgres container");

    // Build connection string
    let port = container
        .get_host_port_ipv4(5432)
        .await
        .expect("Failed to get port");
    let database_url = format!("postgres://postgres:postgres@127.0.0.1:{}/postgres", port);

    // Create connection pool
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to test database");

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    // Create GraphQL schema
    let schema = build_schema();

    // Create test config
    let config = Config {
        database_url,
        database_max_connections: 5,
        jwt_secret: "test_secret_key_for_testing_only_at_least_32_chars".to_string(),
        server_host: "127.0.0.1".to_string(),
        server_port: 8080,
        enable_playground: false,
        cors_origins: vec!["http://localhost:3000".to_string()],
    };

    TestContext {
        pool,
        schema,
        config,
        _container: container,
    }
}
