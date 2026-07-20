use async_graphql::ErrorExtensions;
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
        let code = match &self {
            AppError::Unauthorized(_) | AppError::AuthenticationFailed(_) | AppError::InvalidCredentials | AppError::JwtError(_) => {
                Some("UNAUTHORIZED")
            }
            _ => None,
        };
        let message = self.to_string();
        match code {
            Some(code) => async_graphql::Error::new(message).extend_with(|_, e| {
                e.set("code", code);
            }),
            None => async_graphql::Error::new(message),
        }
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
