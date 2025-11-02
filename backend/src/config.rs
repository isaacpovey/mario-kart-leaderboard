use crate::error::{AppError, Result};
use shuttle_runtime::SecretStore;

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub server_host: String,
    pub server_port: u16,
    pub enable_playground: bool,
    pub cors_origins: Vec<String>,
}

impl Config {
    pub fn from_env(secrets: SecretStore) -> Result<Self> {


        // JWT_SECRET is mandatory for security - fail fast if not set
        let jwt_secret =
            secrets.get("JWT_SECRET").ok_or(AppError::EnvVar(std::env::VarError::NotPresent))?;

        if jwt_secret.len() < 32 {
            return Err(AppError::InvalidInput(
                "JWT_SECRET must be at least 32 characters long for security".to_string(),
            ));
        }

        let server_port = secrets.get("SERVER_PORT")
            .unwrap_or_else(|| "8080".to_string())
            .parse()
            .map_err(|_| AppError::InvalidInput("SERVER_PORT must be a valid u16".to_string()))?;

        // Parse CORS origins from comma-separated env var
        let cors_origins = secrets.get("CORS_ORIGINS")
            .unwrap_or_else(|| "http://localhost:5174".to_string())
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        Ok(Self {
            database_url: secrets.get("DATABASE_URL").unwrap_or_else(|| {
                "postgresql://postgres:password@localhost/mario_kart".to_string()
            }),
            jwt_secret,
            server_host: secrets.get("SERVER_HOST").unwrap_or_else(|| "0.0.0.0".to_string()),
            server_port,
            enable_playground: secrets.get("ENABLE_PLAYGROUND")
                .unwrap_or_else(|| "false".to_string())
                == "true",
            cors_origins,
        })
    }

    pub fn server_addr(&self) -> String {
        format!("{}:{}", self.server_host, self.server_port)
    }
}
