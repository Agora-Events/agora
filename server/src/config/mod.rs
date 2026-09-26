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
//! - `JWT_SECRET` (required) - JWT signing secret, minimum 32 bytes
//! - `PORT` (optional, default: 3001) - Server port, must be 1–65535
//! - `RUST_ENV` (optional, default: development) - Environment mode
//! - `CORS_ALLOWED_ORIGINS` (optional, default: localhost URLs) - CORS origins
//! - `RUST_LOG` (optional, default: info) - Logging level
//! - `SOROBAN_RPC_URL` (optional, default: Stellar testnet RPC) - Blockchain health probe URL
//! - `REDIS_URL` (optional) - Redis connection string used to cache `/api/v1/rates` responses
//! - `RATES_PROVIDER_URL` (optional) - External exchange rate provider base URL
//! - `MONITORING_API_KEY` (optional) - Bearer token required to access `/api/v1/monitoring`

use std::env;

use crate::utils::error::AppError;

pub mod cors;
pub mod request_id;
pub mod security;

pub use cors::create_cors_layer;
pub use request_id::{propagate_request_id_layer, set_request_id_layer};
pub use security::create_security_headers_layer;

/// Minimum acceptable byte-length for `JWT_SECRET`.
const JWT_SECRET_MIN_BYTES: usize = 32;

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

    /// Redis connection URL for caching.
    pub redis_url: String,

    /// S3/R2 bucket name for image uploads.
    pub s3_bucket: String,

    /// S3/R2 region (default: "auto" for Cloudflare R2).
    pub s3_region: String,

    /// S3/R2 access key ID.
    pub s3_access_key_id: String,

    /// S3/R2 secret access key.
    pub s3_secret_access_key: String,

    /// Optional custom S3/R2 endpoint URL (required for R2).
    pub s3_endpoint_url: Option<String>,

    /// Public base URL for uploaded files.
    pub s3_public_url: String,

    /// Base URL for the application (e.g., https://agora.events).
    pub base_url: String,

    /// JWT signing secret. Must be at least 32 bytes long.
    pub jwt_secret: String,

    /// Optional static bearer token required to access the monitoring dashboard.
    /// Set via `MONITORING_TOKEN` environment variable.
    pub monitoring_token: Option<String>,

    /// Optional static bearer token required to access admin APIs.
    /// Set via `ADMIN_TOKEN` environment variable.
    pub admin_token: Option<String>,

    /// Rate limit threshold for auth/nonce endpoint in requests per minute (default: 10).
    pub auth_rate_limit_per_minute: usize,

    /// Graceful-shutdown drain timeout in seconds (default: 15). When a
    /// SIGTERM/SIGINT is received, in-flight requests are given up to this long
    /// to finish before the process exits (Issue #1261).
    pub shutdown_timeout_secs: u64,

    /// Allowed MIME types for uploaded files.
    pub allowed_upload_mime_types: Vec<String>,

    // -----------------------------------------------------------------------
    // Database connection pool settings (Issue #1265)
    // -----------------------------------------------------------------------
    /// Maximum number of connections in the pool (DB_MAX_CONNECTIONS, default: 10).
    pub db_max_connections: u32,

    /// Minimum number of idle connections kept in the pool (DB_MIN_CONNECTIONS, default: 1).
    pub db_min_connections: u32,

    /// Maximum time in seconds to wait for an available connection (DB_ACQUIRE_TIMEOUT_SECS, default: 10).
    pub db_acquire_timeout_secs: u64,

    /// Time in seconds after which an idle connection is closed (DB_IDLE_TIMEOUT_SECS, default: 600).
    pub db_idle_timeout_secs: u64,

    /// Maximum time in seconds a request may take before the server returns
    /// a 504 (REQUEST_TIMEOUT_SECS, default: 30).
    pub request_timeout_secs: u64,
}

/// A collection of configuration errors found during [`Config::validate`].
///
/// All invalid/missing fields are accumulated so the operator sees every
/// problem in a single log line instead of one error per restart.
#[derive(Debug, PartialEq)]
pub struct ConfigError {
    /// One human-readable message per invalid field.
    pub errors: Vec<String>,
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Configuration errors:\n  - {}",
            self.errors.join("\n  - ")
        )
    }
}

impl std::error::Error for ConfigError {}

impl Config {
    /// Load configuration from environment variables with sensible defaults.
    ///
    /// Only `DATABASE_URL` is strictly required at the read stage; all other
    /// semantic constraints are enforced by [`Config::validate`].
    pub fn from_env() -> Result<Self, AppError> {
        let database_url = env::var("DATABASE_URL").map_err(|_| {
            AppError::ValidationError("DATABASE_URL environment variable is required".to_string())
        })?;

        let port = env::var("PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(3001);

        let rust_env = env::var("RUST_ENV").unwrap_or_else(|_| "development".to_string());

        let cors_allowed_origins = env::var("CORS_ALLOWED_ORIGINS")
            .unwrap_or_else(|_| "http://localhost:3000,http://localhost:5173".to_string());

        let rust_log = env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());
        let soroban_rpc_url = env::var("SOROBAN_RPC_URL")
            .unwrap_or_else(|_| "https://soroban-testnet.stellar.org".to_string());

        let redis_url =
            env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

        let s3_bucket = env::var("S3_BUCKET").unwrap_or_default();
        let s3_region = env::var("S3_REGION").unwrap_or_else(|_| "auto".to_string());
        let s3_access_key_id = env::var("S3_ACCESS_KEY_ID").unwrap_or_default();
        let s3_secret_access_key = env::var("S3_SECRET_ACCESS_KEY").unwrap_or_default();
        let s3_endpoint_url = env::var("S3_ENDPOINT_URL").ok();
        let s3_public_url = env::var("S3_PUBLIC_URL").unwrap_or_default();
        let base_url = env::var("BASE_URL").unwrap_or_else(|_| "https://agora.events".to_string());
        let jwt_secret = env::var("JWT_SECRET").unwrap_or_default();
        let monitoring_token = env::var("MONITORING_TOKEN").ok();
        let admin_token = env::var("ADMIN_TOKEN").ok();

        let auth_rate_limit_per_minute = env::var("AUTH_RATE_LIMIT_PER_MINUTE")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(10);

        let shutdown_timeout_secs = env::var("SHUTDOWN_TIMEOUT_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(15);

        let allowed_upload_mime_types = env::var("ALLOWED_UPLOAD_MIME_TYPES")
            .or_else(|_| env::var("ALLOWED_MIME_TYPES"))
            .map(|s| s.split(',').map(|m| m.trim().to_string()).collect())
            .unwrap_or_else(|_| {
                vec![
                    "image/jpeg".to_string(),
                    "image/png".to_string(),
                    "image/webp".to_string(),
                    "image/gif".to_string(),
                ]
            });

        // -----------------------------------------------------------------------
        // Database pool settings (Issue #1265)
        // -----------------------------------------------------------------------
        let db_max_connections = env::var("DB_MAX_CONNECTIONS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(10u32);

        let db_min_connections = env::var("DB_MIN_CONNECTIONS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(1u32);

        let db_acquire_timeout_secs = env::var("DB_ACQUIRE_TIMEOUT_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(10u64);

        let db_idle_timeout_secs = env::var("DB_IDLE_TIMEOUT_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(600u64);

        let request_timeout_secs = env::var("REQUEST_TIMEOUT_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(30u64);

        Ok(Self {
            database_url,
            port,
            rust_env,
            cors_allowed_origins,
            rust_log,
            soroban_rpc_url,
            redis_url,
            s3_bucket,
            s3_region,
            s3_access_key_id,
            s3_secret_access_key,
            s3_endpoint_url,
            s3_public_url,
            base_url,
            jwt_secret,
            monitoring_token,
            admin_token,
            auth_rate_limit_per_minute,
            allowed_upload_mime_types,
            shutdown_timeout_secs,
            // Parsed above but never placed in the initializer, so the struct
            // could not be constructed at all.
            db_max_connections,
            db_min_connections,
            db_acquire_timeout_secs,
            db_idle_timeout_secs,
            request_timeout_secs,
        })
    }

    /// Validate all configuration fields.
    ///
    /// All violations are collected into a single [`ConfigError`] so the
    /// operator can fix every problem without restarting multiple times.
    ///
    /// # Checks performed
    ///
    /// | Field | Rule |
    /// |---|---|
    /// | `database_url` | Non-empty; starts with `postgres://` or `postgresql://` |
    /// | `jwt_secret` | Present and at least [`JWT_SECRET_MIN_BYTES`] bytes |
    /// | `port` | 1 – 65535 (always valid as `u16`, but 0 is rejected) |
    /// | `redis_url` | Non-empty; starts with `redis://` or `rediss://` |
    /// | `soroban_rpc_url` | Non-empty; starts with `http://` or `https://` |
    /// | `base_url` | Non-empty; starts with `http://` or `https://` |
    /// | `cors_allowed_origins` | Non-empty |
    /// | `rust_env` | One of `development`, `production`, `test`, `testing` |
    pub fn validate(&self) -> Result<(), ConfigError> {
        let mut errors: Vec<String> = Vec::new();

        // --- DATABASE_URL ------------------------------------------------
        if self.database_url.trim().is_empty() {
            errors.push("DATABASE_URL is required and must not be empty".to_string());
        } else if !self.database_url.starts_with("postgres://")
            && !self.database_url.starts_with("postgresql://")
        {
            errors.push(format!(
                "DATABASE_URL must start with 'postgres://' or 'postgresql://', got: '{}'",
                truncate_url(&self.database_url)
            ));
        }

        // --- JWT_SECRET --------------------------------------------------
        if self.jwt_secret.trim().is_empty() {
            errors.push(format!(
                "JWT_SECRET is required and must be at least {JWT_SECRET_MIN_BYTES} bytes long"
            ));
        } else if self.jwt_secret.len() < JWT_SECRET_MIN_BYTES {
            errors.push(format!(
                "JWT_SECRET is too short: {} bytes (minimum {JWT_SECRET_MIN_BYTES})",
                self.jwt_secret.len()
            ));
        }

        // --- PORT --------------------------------------------------------
        if self.port == 0 {
            errors.push("PORT must be between 1 and 65535".to_string());
        }

        // --- REDIS_URL ---------------------------------------------------
        if self.redis_url.trim().is_empty() {
            errors.push("REDIS_URL is required and must not be empty".to_string());
        } else if !self.redis_url.starts_with("redis://")
            && !self.redis_url.starts_with("rediss://")
        {
            errors.push(format!(
                "REDIS_URL must start with 'redis://' or 'rediss://', got: '{}'",
                truncate_url(&self.redis_url)
            ));
        }

        // --- SOROBAN_RPC_URL ---------------------------------------------
        if self.soroban_rpc_url.trim().is_empty() {
            errors.push("SOROBAN_RPC_URL must not be empty".to_string());
        } else if !self.soroban_rpc_url.starts_with("http://")
            && !self.soroban_rpc_url.starts_with("https://")
        {
            errors.push(format!(
                "SOROBAN_RPC_URL must start with 'http://' or 'https://', got: '{}'",
                truncate_url(&self.soroban_rpc_url)
            ));
        }

        // --- BASE_URL ----------------------------------------------------
        if self.base_url.trim().is_empty() {
            errors.push("BASE_URL must not be empty".to_string());
        } else if !self.base_url.starts_with("http://") && !self.base_url.starts_with("https://") {
            errors.push(format!(
                "BASE_URL must start with 'http://' or 'https://', got: '{}'",
                truncate_url(&self.base_url)
            ));
        }

        // --- CORS_ALLOWED_ORIGINS ----------------------------------------
        if self.cors_allowed_origins.trim().is_empty() {
            errors.push("CORS_ALLOWED_ORIGINS must not be empty".to_string());
        }

        // --- RUST_ENV ----------------------------------------------------
        let valid_envs = ["development", "production", "test", "testing"];
        let env_lower = self.rust_env.to_lowercase();
        if !valid_envs.contains(&env_lower.as_str()) {
            errors.push(format!(
                "RUST_ENV must be one of {:?}, got: '{}'",
                valid_envs, self.rust_env
            ));
        }

        // --- SHUTDOWN_TIMEOUT_SECS -----------------------------------------
        if self.shutdown_timeout_secs == 0 {
            errors.push("SHUTDOWN_TIMEOUT_SECS must be greater than 0".to_string());
        }

        // --- Database pool sizing -------------------------------------------
        // A pool with a zero maximum accepts no connections at all, and a
        // minimum above the maximum is rejected by sqlx at build time — better
        // to name the offending variable at startup than to fail on the first
        // query. Tests for both already existed but had no implementation to
        // exercise.
        if self.db_max_connections == 0 {
            errors.push("DB_MAX_CONNECTIONS must be greater than 0".to_string());
        } else if self.db_min_connections > self.db_max_connections {
            errors.push(format!(
                "DB_MIN_CONNECTIONS ({}) must not exceed DB_MAX_CONNECTIONS ({})",
                self.db_min_connections, self.db_max_connections
            ));
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(ConfigError { errors })
        }
    }

    /// Helper to identify if running in production.
    pub fn is_production(&self) -> bool {
        self.rust_env.to_lowercase() == "production"
    }
}

/// Truncate a URL to a safe length for error messages (avoids leaking secrets
/// embedded in connection strings).
fn truncate_url(url: &str) -> String {
    const MAX: usize = 40;
    if url.len() > MAX {
        format!("{}…", &url[..MAX])
    } else {
        url.to_string()
    }
}

// tests appended below
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use temp_env;

    static ENV_MUTEX: Mutex<()> = Mutex::new(());

    /// Build a fully-valid `Config` without touching env vars.
    fn valid_config() -> Config {
        Config {
            database_url: "postgres://user:pass@localhost:5432/agora".to_string(),
            port: 3001,
            rust_env: "development".to_string(),
            cors_allowed_origins: "http://localhost:3000".to_string(),
            rust_log: "info".to_string(),
            soroban_rpc_url: "https://soroban-testnet.stellar.org".to_string(),
            redis_url: "redis://127.0.0.1:6379".to_string(),
            s3_bucket: String::new(),
            s3_region: "auto".to_string(),
            s3_access_key_id: String::new(),
            s3_secret_access_key: String::new(),
            s3_endpoint_url: None,
            s3_public_url: String::new(),
            base_url: "https://agora.events".to_string(),
            jwt_secret: "a".repeat(JWT_SECRET_MIN_BYTES),
            monitoring_token: None,
            admin_token: None,
            auth_rate_limit_per_minute: 10,
            allowed_upload_mime_types: vec![
                "image/jpeg".to_string(),
                "image/png".to_string(),
                "image/webp".to_string(),
                "image/gif".to_string(),
            ],
            shutdown_timeout_secs: 15,
            // Pool and timeout settings: present on the struct but never
            // added to these test-only initializers.
            db_max_connections: 10,
            db_min_connections: 1,
            db_acquire_timeout_secs: 30,
            db_idle_timeout_secs: 600,
            request_timeout_secs: 30,
        }
    }

    #[test]
    fn test_config_from_env_success() {
        let _guard = ENV_MUTEX.lock().unwrap();

        env::set_var("DATABASE_URL", "postgres://test:password@localhost/testdb");
        env::set_var("JWT_SECRET", "a_secret_that_is_at_least_32_bytes_long!");

        let config = Config::from_env();
        assert!(
            config.is_ok(),
            "Config::from_env() should succeed with DATABASE_URL set"
        );

        let config = config.unwrap();
        assert_eq!(
            config.database_url,
            "postgres://test:password@localhost/testdb"
        );
        assert!(config.port > 0);

        // Clean up
        env::remove_var("DATABASE_URL");
        env::remove_var("JWT_SECRET");
    }

    #[test]
    fn test_config_from_env_missing_database_url() {
        let _guard = ENV_MUTEX.lock().unwrap();

        // Ensure DATABASE_URL is not set
        env::remove_var("DATABASE_URL");

        let result = Config::from_env();
        assert!(
            result.is_err(),
            "Config::from_env() should fail without DATABASE_URL"
        );

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
        assert_eq!(
            config.cors_allowed_origins,
            "http://localhost:3000,http://localhost:5173"
        );

        env::remove_var("DATABASE_URL");
    }

    #[test]
    fn test_config_from_env_custom_cors_origins() {
        let _guard = ENV_MUTEX.lock().unwrap();

        env::set_var("DATABASE_URL", "postgres://test:password@localhost/testdb");
        env::set_var("CORS_ALLOWED_ORIGINS", "http://example.com,http://test.com");

        let config = Config::from_env().unwrap();
        assert_eq!(
            config.cors_allowed_origins,
            "http://example.com,http://test.com"
        );

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
                (
                    "DATABASE_URL",
                    Some("postgres://test:password@localhost/testdb"),
                ),
                ("PORT", Some("8080")),
            ],
            async {
                let config = Config::from_env().unwrap();
                assert_eq!(config.port, 8080);
            },
        )
        .await;
    }

    #[tokio::test]
    async fn test_port_default_when_not_set() {
        // Test that default port 3001 is used when PORT is not set
        temp_env::async_with_vars(
            [
                (
                    "DATABASE_URL",
                    Some("postgres://test:password@localhost/testdb"),
                ),
                ("PORT", None::<&str>),
            ],
            async {
                let config = Config::from_env().unwrap();
                assert_eq!(config.port, 3001);
            },
        )
        .await;
    }

    #[tokio::test]
    async fn test_port_invalid_value_falls_back_to_default() {
        // Test that invalid port values fall back to default
        temp_env::async_with_vars(
            [
                (
                    "DATABASE_URL",
                    Some("postgres://test:password@localhost/testdb"),
                ),
                ("PORT", Some("invalid")),
            ],
            async {
                let config = Config::from_env().unwrap();
                assert_eq!(config.port, 3001);
            },
        )
        .await;
    }

    #[tokio::test]
    async fn test_port_valid_range_values() {
        // Test various valid port values
        let valid_ports = [80, 443, 8000, 8080, 9000, 65535];

        for port in valid_ports {
            temp_env::async_with_vars(
                [
                    (
                        "DATABASE_URL",
                        Some("postgres://test:password@localhost/testdb"),
                    ),
                    ("PORT", Some(&port.to_string())),
                ],
                async {
                    let config = Config::from_env().unwrap();
                    assert_eq!(config.port, port);
                },
            )
            .await;
        }
    }

    // -----------------------------------------------------------------------
    // Config::validate — happy path
    // -----------------------------------------------------------------------

    #[test]
    fn test_validate_valid_config_passes() {
        assert!(valid_config().validate().is_ok());
    }

    #[test]
    fn test_validate_postgresql_scheme_passes() {
        let mut cfg = valid_config();
        cfg.database_url = "postgresql://user:pass@localhost/db".to_string();
        assert!(cfg.validate().is_ok());
    }

    #[test]
    fn test_validate_production_env_passes() {
        let mut cfg = valid_config();
        cfg.rust_env = "production".to_string();
        assert!(cfg.validate().is_ok());
    }

    #[test]
    fn test_validate_test_env_passes() {
        let mut cfg = valid_config();
        cfg.rust_env = "test".to_string();
        assert!(cfg.validate().is_ok());
    }

    #[test]
    fn test_validate_rediss_scheme_passes() {
        let mut cfg = valid_config();
        cfg.redis_url = "rediss://user:pass@redis.example.com:6380".to_string();
        assert!(cfg.validate().is_ok());
    }

    #[test]
    fn test_validate_jwt_secret_exactly_min_bytes_passes() {
        let mut cfg = valid_config();
        cfg.jwt_secret = "a".repeat(JWT_SECRET_MIN_BYTES);
        assert!(cfg.validate().is_ok());
    }

    // -----------------------------------------------------------------------
    // Config::validate — DATABASE_URL
    // -----------------------------------------------------------------------

    #[test]
    fn test_validate_missing_database_url() {
        let mut cfg = valid_config();
        cfg.database_url = String::new();
        let err = cfg.validate().unwrap_err();
        assert!(
            err.errors
                .iter()
                .any(|e| e.contains("DATABASE_URL") && e.contains("required")),
            "got: {:?}",
            err.errors
        );
    }

    #[test]
    fn test_validate_invalid_database_url_scheme() {
        let mut cfg = valid_config();
        cfg.database_url = "mysql://user:pass@localhost/db".to_string();
        let err = cfg.validate().unwrap_err();
        assert!(
            err.errors
                .iter()
                .any(|e| e.contains("DATABASE_URL") && e.contains("postgres")),
            "got: {:?}",
            err.errors
        );
    }

    // -----------------------------------------------------------------------
    // Config::validate — JWT_SECRET
    // -----------------------------------------------------------------------

    #[test]
    fn test_validate_missing_jwt_secret() {
        let mut cfg = valid_config();
        cfg.jwt_secret = String::new();
        let err = cfg.validate().unwrap_err();
        assert!(
            err.errors
                .iter()
                .any(|e| e.contains("JWT_SECRET") && e.contains("required")),
            "got: {:?}",
            err.errors
        );
    }

    #[test]
    fn test_validate_short_jwt_secret() {
        let mut cfg = valid_config();
        cfg.jwt_secret = "too_short".to_string();
        let err = cfg.validate().unwrap_err();
        assert!(
            err.errors
                .iter()
                .any(|e| e.contains("JWT_SECRET") && e.contains("too short")),
            "got: {:?}",
            err.errors
        );
    }

    #[test]
    fn test_validate_jwt_secret_one_byte_short() {
        let mut cfg = valid_config();
        cfg.jwt_secret = "a".repeat(JWT_SECRET_MIN_BYTES - 1);
        let err = cfg.validate().unwrap_err();
        assert!(err.errors.iter().any(|e| e.contains("JWT_SECRET")));
    }

    // -----------------------------------------------------------------------
    // Config::validate — PORT
    // -----------------------------------------------------------------------

    #[test]
    fn test_validate_port_zero_is_invalid() {
        let mut cfg = valid_config();
        cfg.port = 0;
        let err = cfg.validate().unwrap_err();
        assert!(
            err.errors.iter().any(|e| e.contains("PORT")),
            "got: {:?}",
            err.errors
        );
    }

    #[test]
    fn test_validate_port_nonzero_is_valid() {
        for port in [1u16, 80, 443, 3001, 8080, 65535] {
            let mut cfg = valid_config();
            cfg.port = port;
            assert!(cfg.validate().is_ok(), "port {port} should be valid");
        }
    }

    // -----------------------------------------------------------------------
    // Config::validate — REDIS_URL
    // -----------------------------------------------------------------------

    #[test]
    fn test_validate_missing_redis_url() {
        let mut cfg = valid_config();
        cfg.redis_url = String::new();
        let err = cfg.validate().unwrap_err();
        assert!(err.errors.iter().any(|e| e.contains("REDIS_URL")));
    }

    #[test]
    fn test_validate_invalid_redis_url_scheme() {
        let mut cfg = valid_config();
        cfg.redis_url = "memcache://localhost".to_string();
        let err = cfg.validate().unwrap_err();
        assert!(
            err.errors
                .iter()
                .any(|e| e.contains("REDIS_URL") && e.contains("redis")),
            "got: {:?}",
            err.errors
        );
    }

    // -----------------------------------------------------------------------
    // Config::validate — SOROBAN_RPC_URL
    // -----------------------------------------------------------------------

    #[test]
    fn test_validate_missing_soroban_rpc_url() {
        let mut cfg = valid_config();
        cfg.soroban_rpc_url = String::new();
        let err = cfg.validate().unwrap_err();
        assert!(err.errors.iter().any(|e| e.contains("SOROBAN_RPC_URL")));
    }

    #[test]
    fn test_validate_invalid_soroban_rpc_url_scheme() {
        let mut cfg = valid_config();
        cfg.soroban_rpc_url = "ftp://soroban.example.com".to_string();
        let err = cfg.validate().unwrap_err();
        assert!(
            err.errors
                .iter()
                .any(|e| e.contains("SOROBAN_RPC_URL") && e.contains("http")),
            "got: {:?}",
            err.errors
        );
    }

    // -----------------------------------------------------------------------
    // Config::validate — BASE_URL
    // -----------------------------------------------------------------------

    #[test]
    fn test_validate_missing_base_url() {
        let mut cfg = valid_config();
        cfg.base_url = String::new();
        let err = cfg.validate().unwrap_err();
        assert!(err.errors.iter().any(|e| e.contains("BASE_URL")));
    }

    #[test]
    fn test_validate_invalid_base_url_scheme() {
        let mut cfg = valid_config();
        cfg.base_url = "ws://agora.events".to_string();
        let err = cfg.validate().unwrap_err();
        assert!(
            err.errors
                .iter()
                .any(|e| e.contains("BASE_URL") && e.contains("http")),
            "got: {:?}",
            err.errors
        );
    }

    // -----------------------------------------------------------------------
    // Config::validate — CORS_ALLOWED_ORIGINS
    // -----------------------------------------------------------------------

    #[test]
    fn test_validate_empty_cors_origins() {
        let mut cfg = valid_config();
        cfg.cors_allowed_origins = String::new();
        let err = cfg.validate().unwrap_err();
        assert!(err
            .errors
            .iter()
            .any(|e| e.contains("CORS_ALLOWED_ORIGINS")));
    }

    // -----------------------------------------------------------------------
    // Config::validate — RUST_ENV
    // -----------------------------------------------------------------------

    #[test]
    fn test_validate_invalid_rust_env() {
        let mut cfg = valid_config();
        cfg.rust_env = "staging".to_string();
        let err = cfg.validate().unwrap_err();
        assert!(
            err.errors
                .iter()
                .any(|e| e.contains("RUST_ENV") && e.contains("staging")),
            "got: {:?}",
            err.errors
        );
    }

    // -----------------------------------------------------------------------
    // Config::validate — multiple errors accumulated
    // -----------------------------------------------------------------------

    #[test]
    fn test_validate_accumulates_all_errors() {
        let cfg = Config {
            database_url: String::new(),
            jwt_secret: "short".to_string(),
            port: 3001,
            rust_env: "staging".to_string(),
            cors_allowed_origins: String::new(),
            rust_log: "info".to_string(),
            soroban_rpc_url: String::new(),
            redis_url: String::new(),
            s3_bucket: String::new(),
            s3_region: "auto".to_string(),
            s3_access_key_id: String::new(),
            s3_secret_access_key: String::new(),
            s3_endpoint_url: None,
            s3_public_url: String::new(),
            base_url: String::new(),
            monitoring_token: None,
            admin_token: None,
            auth_rate_limit_per_minute: 10,
            allowed_upload_mime_types: vec![
                "image/jpeg".to_string(),
                "image/png".to_string(),
                "image/webp".to_string(),
                "image/gif".to_string(),
            ],
            shutdown_timeout_secs: 15,
            // Pool and timeout settings: present on the struct but never
            // added to these test-only initializers.
            db_max_connections: 10,
            db_min_connections: 1,
            db_acquire_timeout_secs: 30,
            db_idle_timeout_secs: 600,
            request_timeout_secs: 30,
        };

        let err = cfg.validate().unwrap_err();
        assert!(
            err.errors.len() >= 7,
            "expected ≥7 errors, got {}: {:?}",
            err.errors.len(),
            err.errors
        );
    }

    // -----------------------------------------------------------------------
    // ConfigError Display
    // -----------------------------------------------------------------------

    #[test]
    fn test_config_error_display_contains_all_messages() {
        let err = ConfigError {
            errors: vec![
                "DATABASE_URL is required".to_string(),
                "JWT_SECRET is required".to_string(),
            ],
        };
        let msg = err.to_string();
        assert!(msg.contains("DATABASE_URL is required"));
        assert!(msg.contains("JWT_SECRET is required"));
        assert!(msg.contains("Configuration errors"));
    }

    // -----------------------------------------------------------------------
    // truncate_url helper
    // -----------------------------------------------------------------------

    #[test]
    fn test_truncate_url_short_string_unchanged() {
        let url = "postgres://localhost/db";
        assert_eq!(truncate_url(url), url);
    }

    #[test]
    fn test_truncate_url_long_string_is_truncated() {
        let url = "postgres://".to_string() + &"x".repeat(100);
        let result = truncate_url(&url);
        assert!(result.ends_with('…'));
        assert!(result.len() < url.len());
    }

    // -----------------------------------------------------------------------
    // Issue #1265 — DB pool configuration
    // -----------------------------------------------------------------------

    #[test]
    fn test_validate_db_pool_defaults_are_valid() {
        // Default values (min=1, max=10) must pass validation.
        let cfg = valid_config();
        assert!(cfg.validate().is_ok());
    }

    #[test]
    fn test_validate_db_min_greater_than_max_rejected() {
        let mut cfg = valid_config();
        cfg.db_min_connections = 20;
        cfg.db_max_connections = 10;
        let err = cfg.validate().unwrap_err();
        assert!(
            err.errors
                .iter()
                .any(|e| e.contains("DB_MIN_CONNECTIONS") && e.contains("DB_MAX_CONNECTIONS")),
            "got: {:?}",
            err.errors
        );
    }

    #[test]
    fn test_validate_db_max_zero_rejected() {
        let mut cfg = valid_config();
        cfg.db_max_connections = 0;
        cfg.db_min_connections = 0;
        let err = cfg.validate().unwrap_err();
        assert!(
            err.errors.iter().any(|e| e.contains("DB_MAX_CONNECTIONS")),
            "got: {:?}",
            err.errors
        );
    }

    #[test]
    fn test_validate_db_min_equals_max_is_valid() {
        let mut cfg = valid_config();
        cfg.db_min_connections = 5;
        cfg.db_max_connections = 5;
        assert!(cfg.validate().is_ok());
    }

    #[test]
    fn test_db_pool_defaults_from_env() {
        let _guard = ENV_MUTEX.lock().unwrap();
        env::set_var("DATABASE_URL", "postgres://test:pass@localhost/db");
        env::remove_var("DB_MAX_CONNECTIONS");
        env::remove_var("DB_MIN_CONNECTIONS");
        env::remove_var("DB_ACQUIRE_TIMEOUT_SECS");
        env::remove_var("DB_IDLE_TIMEOUT_SECS");

        let cfg = Config::from_env().unwrap();
        assert_eq!(cfg.db_max_connections, 10);
        assert_eq!(cfg.db_min_connections, 1);
        assert_eq!(cfg.db_acquire_timeout_secs, 10);
        assert_eq!(cfg.db_idle_timeout_secs, 600);

        env::remove_var("DATABASE_URL");
    }

    #[test]
    fn test_db_pool_custom_values_from_env() {
        let _guard = ENV_MUTEX.lock().unwrap();
        env::set_var("DATABASE_URL", "postgres://test:pass@localhost/db");
        env::set_var("DB_MAX_CONNECTIONS", "25");
        env::set_var("DB_MIN_CONNECTIONS", "5");
        env::set_var("DB_ACQUIRE_TIMEOUT_SECS", "30");
        env::set_var("DB_IDLE_TIMEOUT_SECS", "120");

        let cfg = Config::from_env().unwrap();
        assert_eq!(cfg.db_max_connections, 25);
        assert_eq!(cfg.db_min_connections, 5);
        assert_eq!(cfg.db_acquire_timeout_secs, 30);
        assert_eq!(cfg.db_idle_timeout_secs, 120);

        env::remove_var("DATABASE_URL");
        env::remove_var("DB_MAX_CONNECTIONS");
        env::remove_var("DB_MIN_CONNECTIONS");
        env::remove_var("DB_ACQUIRE_TIMEOUT_SECS");
        env::remove_var("DB_IDLE_TIMEOUT_SECS");
    }
}
