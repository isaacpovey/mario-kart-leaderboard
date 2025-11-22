use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("JWT error: {0}")]
    JwtError(#[from] jsonwebtoken::errors::Error),

    #[error("Password hashing error: {0}")]
    PasswordHashError(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Migration error: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),

    #[error("Environment variable error: {0}")]
    EnvVar(#[from] std::env::VarError),

    #[error("Internal server error: {0}")]
    Internal(String),

    #[error("Telemetry error: {0}")]
    Telemetry(String),
}

pub trait IntoGraphQLError {
    fn into_graphql_error(self) -> async_graphql::Error;
}

impl IntoGraphQLError for AppError {
    fn into_graphql_error(self) -> async_graphql::Error {
        async_graphql::Error::new(self.to_string())
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
