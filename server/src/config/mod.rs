use std::env;

pub mod cors;
pub mod security;

pub use cors::create_cors_layer;
pub use security::create_security_headers_layer;

/// Application configuration loaded from environment variables.
#[derive(Debug, Clone)]
pub struct Config {
    /// Database connection URL.
    pub database_url: String,

    /// Server port (default: 3001).
    pub port: u16,

    /// Environment (development, production, testing).
    pub rust_env: String,

    /// Comma-separated list of allowed origins for CORS.
    pub cors_allowed_origins: String,

    /// Logging configuration (RUST_LOG).
    pub rust_log: String,
}

impl Config {
    /// Load configuration from environment variables with sensible defaults.
    pub fn from_env() -> Self {
        Self {
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://localhost/agora".to_string()),

            port: env::var("PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(3001),

            rust_env: env::var("RUST_ENV").unwrap_or_else(|_| "development".to_string()),

            cors_allowed_origins: env::var("CORS_ALLOWED_ORIGINS")
                .unwrap_or_else(|_| "http://localhost:3000,http://localhost:5173".to_string()),

            rust_log: env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
        }
    }

    /// Helper to identify if running in production.
    pub fn is_production(&self) -> bool {
        self.rust_env.to_lowercase() == "production"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_from_env_defaults() {
        // Ensure that clearing environment variables doesn't break initialization.
        // In practice we can't easily clear all global env in parallel tests,
        // but we can verify that the default values are correct if variables are unset.

        // We'll just test that from_env() at least works and has expected structure.
        let config = Config::from_env();
        assert!(config.port > 0);
    }

    #[test]
    fn test_is_production() {
        let mut config = Config::from_env();
        config.rust_env = "production".into();
        assert!(config.is_production());

        config.rust_env = "development".into();
        assert!(!config.is_production());
    }
}
