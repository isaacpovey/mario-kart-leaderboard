use crate::error::{AppError, Result};
use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub database_max_connections: u32,
    pub jwt_secret: String,
    pub server_host: String,
    pub server_port: u16,
    pub enable_playground: bool,
    pub cors_origins: Vec<String>,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenv::dotenv().ok();

        // JWT_SECRET is mandatory for security - fail fast if not set
        let jwt_secret =
            env::var("JWT_SECRET").map_err(|_| AppError::EnvVar(std::env::VarError::NotPresent))?;

        if jwt_secret.len() < 32 {
            return Err(AppError::InvalidInput(
                "JWT_SECRET must be at least 32 characters long for security".to_string(),
            ));
        }

        let server_port = env::var("SERVER_PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse()
            .map_err(|_| AppError::InvalidInput("SERVER_PORT must be a valid u16".to_string()))?;

        // Parse CORS origins from comma-separated env var
        let cors_origins = env::var("CORS_ORIGINS")
            .unwrap_or_else(|_| "http://localhost:5174".to_string())
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let database_max_connections = env::var("DATABASE_MAX_CONNECTIONS")
            .unwrap_or_else(|_| "10".to_string())
            .parse()
            .map_err(|_| {
                AppError::InvalidInput("DATABASE_MAX_CONNECTIONS must be a valid u32".to_string())
            })?;

        Ok(Self {
            database_url: env::var("DATABASE_URL").unwrap_or_else(|_| {
                "postgresql://postgres:password@localhost/mario_kart".to_string()
            }),
            database_max_connections,
            jwt_secret,
            server_host: env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            server_port,
            enable_playground: env::var("ENABLE_PLAYGROUND")
                .unwrap_or_else(|_| "false".to_string())
                == "true",
            cors_origins,
        })
    }

    pub fn server_addr(&self) -> String {
        format!("{}:{}", self.server_host, self.server_port)
    }
}
