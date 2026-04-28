//! # Configuration Module
//!
//! This module handles application configuration loaded from environment variables.
//! It provides a centralized configuration structure with sensible defaults
//! and validation for required settings.
//!
//! ## Sub-modules
//!
//! - [`cors`] - Cross-Origin Resource Sharing configuration
//! - [`request_id`] - Request ID middleware configuration
//! - [`security`] - Security headers configuration
//!
//! ## Environment Variables
//!
//! The following environment variables are supported:
//! - `DATABASE_URL` (required) - PostgreSQL connection string
//! - `PORT` (optional, default: 3001) - Server port
//! - `RUST_ENV` (optional, default: development) - Environment mode
//! - `CORS_ALLOWED_ORIGINS` (optional, default: localhost URLs) - CORS origins
//! - `RUST_LOG` (optional, default: info) - Logging level
//! - `SOROBAN_RPC_URL` (optional, default: Stellar testnet RPC) - Blockchain health probe URL

use std::env;

use crate::utils::error::AppError;

pub mod cors;
pub mod request_id;
pub mod security;

pub use cors::create_cors_layer;
pub use request_id::{ propagate_request_id_layer, set_request_id_layer };
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

    /// Soroban RPC URL for blockchain connectivity checks.
    pub soroban_rpc_url: String,
}

impl Config {
    /// Load configuration from environment variables with sensible defaults.
    ///
    /// Returns `Result<Self, AppError>` to properly handle missing or invalid
    /// required environment variables.
    pub fn from_env() -> Result<Self, AppError> {
        let database_url = env
            ::var("DATABASE_URL")
            .map_err(|_| {
                AppError::ValidationError(
                    "DATABASE_URL environment variable is required".to_string()
                )
            })?;

        let port = env
            ::var("PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(3001);

        let rust_env = env::var("RUST_ENV").unwrap_or_else(|_| "development".to_string());

        let cors_allowed_origins = env
            ::var("CORS_ALLOWED_ORIGINS")
            .unwrap_or_else(|_| "http://localhost:3000,http://localhost:5173".to_string());

        let rust_log = env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());
        let soroban_rpc_url =
            env::var("SOROBAN_RPC_URL").unwrap_or_else(|_| "https://soroban-testnet.stellar.org".to_string());

        let s3_bucket = env::var("S3_BUCKET").map_err(|_| {
            AppError::ValidationError("S3_BUCKET environment variable is required".to_string())
        })?;
        let s3_region = env::var("S3_REGION").unwrap_or_else(|_| "auto".to_string());
        let s3_access_key_id = env::var("S3_ACCESS_KEY_ID").map_err(|_| {
            AppError::ValidationError(
                "S3_ACCESS_KEY_ID environment variable is required".to_string(),
            )
        })?;
        let s3_secret_access_key = env::var("S3_SECRET_ACCESS_KEY").map_err(|_| {
            AppError::ValidationError(
                "S3_SECRET_ACCESS_KEY environment variable is required".to_string(),
            )
        })?;
        let s3_endpoint_url = env::var("S3_ENDPOINT_URL").ok();
        let s3_public_url = env::var("S3_PUBLIC_URL").map_err(|_| {
            AppError::ValidationError("S3_PUBLIC_URL environment variable is required".to_string())
        })?;

        let s3_bucket = env::var("S3_BUCKET").map_err(|_| {
            AppError::ValidationError("S3_BUCKET environment variable is required".to_string())
        })?;
        let s3_region = env::var("S3_REGION").unwrap_or_else(|_| "auto".to_string());
        let s3_access_key_id = env::var("S3_ACCESS_KEY_ID").map_err(|_| {
            AppError::ValidationError(
                "S3_ACCESS_KEY_ID environment variable is required".to_string(),
            )
        })?;
        let s3_secret_access_key = env::var("S3_SECRET_ACCESS_KEY").map_err(|_| {
            AppError::ValidationError(
                "S3_SECRET_ACCESS_KEY environment variable is required".to_string(),
            )
        })?;
        let s3_endpoint_url = env::var("S3_ENDPOINT_URL").ok();
        let s3_public_url = env::var("S3_PUBLIC_URL").map_err(|_| {
            AppError::ValidationError("S3_PUBLIC_URL environment variable is required".to_string())
        })?;

        Ok(Self {
            database_url,
            port,
            rust_env,
            cors_allowed_origins,
            rust_log,
            soroban_rpc_url,
        })
    }

    /// Helper to identify if running in production.
    pub fn is_production(&self) -> bool {
        self.rust_env.to_lowercase() == "production"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use temp_env;

    // Mutex to prevent environment variable tests from running in parallel
    static ENV_MUTEX: Mutex<()> = Mutex::new(());

    #[test]
    fn test_config_from_env_success() {
        let _guard = ENV_MUTEX.lock().unwrap();

        // Set required environment variable
        env::set_var("DATABASE_URL", "postgres://test:password@localhost/testdb");

        let config = Config::from_env();
        assert!(config.is_ok(), "Config::from_env() should succeed with DATABASE_URL set");

        let config = config.unwrap();
        assert_eq!(config.database_url, "postgres://test:password@localhost/testdb");
        assert!(config.port > 0);

        // Clean up
        env::remove_var("DATABASE_URL");
    }

    #[test]
    fn test_config_from_env_missing_database_url() {
        let _guard = ENV_MUTEX.lock().unwrap();

        // Ensure DATABASE_URL is not set
        env::remove_var("DATABASE_URL");

        let result = Config::from_env();
        assert!(result.is_err(), "Config::from_env() should fail without DATABASE_URL");

        let err = result.unwrap_err();
        assert!(matches!(err, AppError::ValidationError(_)));
        assert!(err.to_string().contains("DATABASE_URL"));
    }

    #[test]
    fn test_config_from_env_default_port() {
        let _guard = ENV_MUTEX.lock().unwrap();

        env::set_var("DATABASE_URL", "postgres://test:password@localhost/testdb");
        env::remove_var("PORT");

        let config = Config::from_env().unwrap();
        assert_eq!(config.port, 3001);

        env::remove_var("DATABASE_URL");
    }

    #[test]
    fn test_config_from_env_custom_port() {
        let _guard = ENV_MUTEX.lock().unwrap();

        env::set_var("DATABASE_URL", "postgres://test:password@localhost/testdb");
        env::set_var("PORT", "8080");

        let config = Config::from_env().unwrap();
        assert_eq!(config.port, 8080);

        env::remove_var("DATABASE_URL");
        env::remove_var("PORT");
    }

    #[test]
    fn test_config_from_env_default_rust_env() {
        let _guard = ENV_MUTEX.lock().unwrap();

        env::set_var("DATABASE_URL", "postgres://test:password@localhost/testdb");
        env::remove_var("RUST_ENV");

        let config = Config::from_env().unwrap();
        assert_eq!(config.rust_env, "development");

        env::remove_var("DATABASE_URL");
    }

    #[test]
    fn test_config_from_env_custom_rust_env() {
        let _guard = ENV_MUTEX.lock().unwrap();

        env::set_var("DATABASE_URL", "postgres://test:password@localhost/testdb");
        env::set_var("RUST_ENV", "production");

        let config = Config::from_env().unwrap();
        assert_eq!(config.rust_env, "production");

        env::remove_var("DATABASE_URL");
        env::remove_var("RUST_ENV");
    }

    #[test]
    fn test_config_from_env_default_cors_origins() {
        let _guard = ENV_MUTEX.lock().unwrap();

        env::set_var("DATABASE_URL", "postgres://test:password@localhost/testdb");
        env::remove_var("CORS_ALLOWED_ORIGINS");

        let config = Config::from_env().unwrap();
        assert_eq!(config.cors_allowed_origins, "http://localhost:3000,http://localhost:5173");

        env::remove_var("DATABASE_URL");
    }

    #[test]
    fn test_config_from_env_custom_cors_origins() {
        let _guard = ENV_MUTEX.lock().unwrap();

        env::set_var("DATABASE_URL", "postgres://test:password@localhost/testdb");
        env::set_var("CORS_ALLOWED_ORIGINS", "http://example.com,http://test.com");

        let config = Config::from_env().unwrap();
        assert_eq!(config.cors_allowed_origins, "http://example.com,http://test.com");

        env::remove_var("DATABASE_URL");
        env::remove_var("CORS_ALLOWED_ORIGINS");
    }

    #[test]
    fn test_config_from_env_default_rust_log() {
        let _guard = ENV_MUTEX.lock().unwrap();

        env::set_var("DATABASE_URL", "postgres://test:password@localhost/testdb");
        env::remove_var("RUST_LOG");

        let config = Config::from_env().unwrap();
        assert_eq!(config.rust_log, "info");

        env::remove_var("DATABASE_URL");
    }

    #[test]
    fn test_config_from_env_custom_rust_log() {
        let _guard = ENV_MUTEX.lock().unwrap();

        env::set_var("DATABASE_URL", "postgres://test:password@localhost/testdb");
        env::set_var("RUST_LOG", "debug");

        let config = Config::from_env().unwrap();
        assert_eq!(config.rust_log, "debug");

        env::remove_var("DATABASE_URL");
        env::remove_var("RUST_LOG");
    }

    #[test]
    fn test_is_production() {
        let _guard = ENV_MUTEX.lock().unwrap();

        env::set_var("DATABASE_URL", "postgres://test:password@localhost/testdb");

        let mut config = Config::from_env().unwrap();
        config.rust_env = "production".into();
        assert!(config.is_production());

        config.rust_env = "development".into();
        assert!(!config.is_production());

        env::remove_var("DATABASE_URL");
    }

    #[tokio::test]
    async fn test_port_from_env_variable() {
        // Test that PORT environment variable is correctly read
        temp_env::async_with_vars(
            [
                ("DATABASE_URL", Some("postgres://test:password@localhost/testdb")),
                ("PORT", Some("8080")),
            ],
            async {
                let config = Config::from_env().unwrap();
                assert_eq!(config.port, 8080);
            }
        ).await;
    }

    #[tokio::test]
    async fn test_port_default_when_not_set() {
        // Test that default port 3001 is used when PORT is not set
        temp_env::async_with_vars(
            [
                ("DATABASE_URL", Some("postgres://test:password@localhost/testdb")),
                ("PORT", None::<&str>),
            ],
            async {
                let config = Config::from_env().unwrap();
                assert_eq!(config.port, 3001);
            }
        ).await;
    }

    #[tokio::test]
    async fn test_port_invalid_value_falls_back_to_default() {
        // Test that invalid port values fall back to default
        temp_env::async_with_vars(
            [
                ("DATABASE_URL", Some("postgres://test:password@localhost/testdb")),
                ("PORT", Some("invalid")),
            ],
            async {
                let config = Config::from_env().unwrap();
                assert_eq!(config.port, 3001);
            }
        ).await;
    }

    #[tokio::test]
    async fn test_port_valid_range_values() {
        // Test various valid port values
        let valid_ports = [80, 443, 8000, 8080, 9000, 65535];

        for port in valid_ports {
            temp_env::async_with_vars(
                [
                    ("DATABASE_URL", Some("postgres://test:password@localhost/testdb")),
                    ("PORT", Some(&port.to_string())),
                ],
                async {
                    let config = Config::from_env().unwrap();
                    assert_eq!(config.port, port);
                }
            ).await;
        }
    }
}
