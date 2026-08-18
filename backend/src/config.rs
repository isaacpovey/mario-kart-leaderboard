use crate::error::{AppError, Result};
use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    /// Dedicated URL for PostgreSQL LISTEN. Falls back to [`Self::database_url`]
    /// when unset. Use this when the query pool goes through a transaction-mode
    /// pooler (e.g. Supavisor on port 6543) that silently drops LISTEN/NOTIFY.
    pub listen_database_url: Option<String>,
    pub database_max_connections: u32,
    pub jwt_secret: String,
    pub server_host: String,
    pub server_port: u16,
    pub enable_playground: bool,
    pub cors_origins: Vec<String>,
    pub otlp_endpoint: Option<String>,
    pub service_name: String,
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
            .unwrap_or_else(|_| "http://localhost:5173".to_string())
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
            listen_database_url: env::var("LISTEN_DATABASE_URL")
                .ok()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty()),
            database_max_connections,
            jwt_secret,
            server_host: env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            server_port,
            enable_playground: env::var("ENABLE_PLAYGROUND")
                .unwrap_or_else(|_| "false".to_string())
                == "true",
            cors_origins,
            otlp_endpoint: env::var("OTEL_EXPORTER_OTLP_ENDPOINT").ok(),
            service_name: env::var("OTEL_SERVICE_NAME")
                .unwrap_or_else(|_| "mario-kart-leaderboard".to_string()),
        })
    }

    pub fn server_addr(&self) -> String {
        format!("{}:{}", self.server_host, self.server_port)
    }

    /// Connection string used by [`crate::services::notification_manager::NotificationManager::start_listener`].
    ///
    /// Prefer `LISTEN_DATABASE_URL` (session-mode / direct Postgres) when the
    /// main `DATABASE_URL` is a transaction-mode pooler that cannot deliver
    /// LISTEN notifications.
    pub fn listen_database_url(&self) -> &str {
        self.listen_database_url
            .as_deref()
            .unwrap_or(&self.database_url)
    }
}

#[cfg(test)]
mod tests {
    use super::Config;

    fn sample_config(listen: Option<&str>) -> Config {
        Config {
            database_url: "postgresql://pooler:6543/mario_kart".to_string(),
            listen_database_url: listen.map(str::to_string),
            database_max_connections: 10,
            jwt_secret: "test_secret_key_for_testing_only_at_least_32_chars".to_string(),
            server_host: "127.0.0.1".to_string(),
            server_port: 8080,
            enable_playground: false,
            cors_origins: vec![],
            otlp_endpoint: None,
            service_name: "test".to_string(),
        }
    }

    #[test]
    fn listen_url_falls_back_to_database_url() {
        let config = sample_config(None);
        assert_eq!(config.listen_database_url(), config.database_url);
    }

    #[test]
    fn listen_url_uses_dedicated_override() {
        let config = sample_config(Some("postgresql://direct:5432/mario_kart"));
        assert_eq!(
            config.listen_database_url(),
            "postgresql://direct:5432/mario_kart"
        );
    }
}
