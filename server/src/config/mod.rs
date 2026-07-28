//! # Configuration Module
//!
//! Loads and **eagerly validates** all application configuration at startup.
//! Every value is checked before the server binds its port so that
//! misconfiguration surfaces immediately with a clear, actionable message
//! rather than causing a cryptic panic at first use.
//!
//! ## Validation strategy
//! - **Required vars** — missing value → hard error with the var name.
//! - **Optional vars with defaults** — present but unparseable → hard error
//!   (e.g. `PORT=abc` instead of silently falling back to 3001).
//! - **Range checks** — `PORT` must be 1–65535; `SOROBAN_START_LEDGER` ≥ 0.
//! - **URL syntax** — `DATABASE_URL`, `REDIS_URL`, `BASE_URL`,
//!   `SOROBAN_RPC_URL`, `S3_ENDPOINT_URL`, and each `CORS_ALLOWED_ORIGINS`
//!   entry are parsed with the `url` crate.
//! - **Incompatible combinations** — production requires a strong
//!   `JWT_SECRET`; S3 credentials must all be present together; in
//!   production all CORS origins must be HTTPS.
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
use url::Url;

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
    /// PostgreSQL connection URL (`DATABASE_URL`).
    pub database_url: String,

    /// TCP port the HTTP server listens on (`PORT`, default 3001).
    pub port: u16,

    /// Deployment environment (`RUST_ENV`, default `"development"`).
    pub rust_env: String,

    /// Comma-separated allowed CORS origins (`CORS_ALLOWED_ORIGINS`).
    pub cors_allowed_origins: String,

    /// Tracing filter string (`RUST_LOG`, default `"info"`).
    pub rust_log: String,

    /// Stellar/Soroban JSON-RPC endpoint (`SOROBAN_RPC_URL`).
    pub soroban_rpc_url: String,

    /// Redis connection URL (`REDIS_URL`, default `redis://127.0.0.1:6379`).
    pub redis_url: String,

    /// S3/R2 bucket name (`S3_BUCKET`).  Empty string means uploads disabled.
    pub s3_bucket: String,

    /// S3/R2 region (`S3_REGION`, default `"auto"`).
    pub s3_region: String,

    /// S3/R2 access-key ID (`S3_ACCESS_KEY_ID`).
    pub s3_access_key_id: String,

    /// S3/R2 secret access key (`S3_SECRET_ACCESS_KEY`).
    pub s3_secret_access_key: String,

    /// Optional custom S3/R2 endpoint URL (`S3_ENDPOINT_URL`).
    pub s3_endpoint_url: Option<String>,

    /// Public base URL for uploaded assets (`S3_PUBLIC_URL`).
    pub s3_public_url: String,

    /// Application base URL (`BASE_URL`, default `https://agora.events`).
    pub base_url: String,

    /// JWT signing secret. Must be at least 32 bytes long.
    pub jwt_secret: String,

    /// Optional static bearer token required to access the monitoring dashboard.
    /// Set via `MONITORING_TOKEN` environment variable.
    pub monitoring_token: Option<String>,

    /// Optional static bearer token required to access admin APIs.
    /// Set via `ADMIN_TOKEN` environment variable.
    pub admin_token: Option<String>,
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
    /// Load and validate all configuration from environment variables.
    ///
    /// Only `DATABASE_URL` is strictly required at the read stage; all other
    /// semantic constraints are enforced by [`Config::validate`].
    pub fn from_env() -> Result<Self, AppError> {
        let mut errors: Vec<String> = Vec::new();

        // ── Required ────────────────────────────────────────────────────────
        let database_url = require_url("DATABASE_URL", &mut errors);

        // ── Port ────────────────────────────────────────────────────────────
        let port = parse_port(&mut errors);

        // ── RUST_ENV ────────────────────────────────────────────────────────
        let rust_env = validate_rust_env(&mut errors);

        // ── RUST_LOG ────────────────────────────────────────────────────────
        let rust_log = validate_rust_log(&mut errors);

        // ── CORS origins ────────────────────────────────────────────────────
        let cors_allowed_origins = env::var("CORS_ALLOWED_ORIGINS")
            .unwrap_or_else(|_| "http://localhost:3000,http://localhost:5173".to_string());

        // ── URLs with defaults ───────────────────────────────────────────────
        let soroban_rpc_url = optional_url(
            "SOROBAN_RPC_URL",
            "https://soroban-testnet.stellar.org",
            &mut errors,
        );

        let redis_url = optional_url("REDIS_URL", "redis://127.0.0.1:6379", &mut errors);

        let base_url = optional_url("BASE_URL", "https://agora.events", &mut errors);

        // ── S3 / R2 ─────────────────────────────────────────────────────────
        let s3_bucket = env::var("S3_BUCKET").unwrap_or_default();
        let s3_region = env::var("S3_REGION").unwrap_or_else(|_| "auto".to_string());
        let s3_access_key_id = env::var("S3_ACCESS_KEY_ID").unwrap_or_default();
        let s3_secret_access_key = env::var("S3_SECRET_ACCESS_KEY").unwrap_or_default();
        let s3_public_url = env::var("S3_PUBLIC_URL").unwrap_or_default();
        let base_url = env::var("BASE_URL").unwrap_or_else(|_| "https://agora.events".to_string());
        let jwt_secret = env::var("JWT_SECRET").unwrap_or_default();
        let monitoring_token = env::var("MONITORING_TOKEN").ok();
        let admin_token = env::var("ADMIN_TOKEN").ok();

        Ok(Self {
            database_url: database_url.unwrap_or_default(),
            port: port.unwrap_or(3001),
            rust_env,
            cors_allowed_origins,
            rust_log,
            soroban_rpc_url: soroban_rpc_url.unwrap_or_default(),
            redis_url: redis_url.unwrap_or_default(),
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

    /// Returns `true` when S3/R2 image uploads are configured.
    pub fn s3_enabled(&self) -> bool {
        !self.s3_bucket.is_empty()
            && !self.s3_access_key_id.is_empty()
            && !self.s3_secret_access_key.is_empty()
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
        }
    }

    #[test]
    fn test_config_from_env_success() {
        let _guard = ENV_MUTEX.lock().unwrap();

        env::set_var("DATABASE_URL", "postgres://test:password@localhost/testdb");
        env::set_var("JWT_SECRET", "a_secret_that_is_at_least_32_bytes_long!");

/// Parse and range-check `PORT`.
fn parse_port(errors: &mut Vec<String>) -> Option<u16> {
    match env::var("PORT") {
        Err(_) => Some(3001), // absent → use default silently
        Ok(v) => match v.trim().parse::<u32>() {
            Err(_) => {
                errors.push(format!(
                    "PORT must be a number between 1 and 65535, got: {v:?}"
                ));
                None
            }
            Ok(0) => {
                errors.push("PORT must be ≥ 1 (port 0 is not bindable).".to_string());
                None
            }
            Ok(n) if n > 65535 => {
                errors.push(format!(
                    "PORT must be ≤ 65535, got: {n}"
                ));
                None
            }
            Ok(n) => Some(n as u16),
        },
    }
}

/// Validate `RUST_ENV` against the known set of values.
fn validate_rust_env(errors: &mut Vec<String>) -> String {
    let value = env::var("RUST_ENV").unwrap_or_else(|_| "development".to_string());
    let lower = value.to_lowercase();
    if !VALID_ENVIRONMENTS.contains(&lower.as_str()) {
        errors.push(format!(
            "RUST_ENV has unrecognised value {value:?}. \
             Valid values: {}.",
            VALID_ENVIRONMENTS.join(", ")
        ));
    }
    lower
}

        // Clean up
        env::remove_var("DATABASE_URL");
        env::remove_var("JWT_SECRET");
    }
    value
}

/// Cross-field / incompatible-combination checks.
#[allow(clippy::too_many_arguments)]
fn cross_validate(
    is_production: bool,
    jwt_secret: &str,
    cors_allowed_origins: &str,
    s3_bucket: &str,
    s3_access_key_id: &str,
    s3_secret_access_key: &str,
    s3_public_url: &str,
    s3_endpoint_url: Option<&str>,
    errors: &mut Vec<String>,
) {
    // 1. Production requires a strong JWT_SECRET.
    if is_production {
        if jwt_secret == "fallback_dev_secret_change_in_prod" {
            errors.push(
                "JWT_SECRET must be set in production — \
                 the built-in fallback secret is not safe for production use."
                    .to_string(),
            );
        } else if jwt_secret.len() < MIN_JWT_SECRET_LEN_PROD {
            errors.push(format!(
                "JWT_SECRET must be at least {MIN_JWT_SECRET_LEN_PROD} characters in production \
                 (current length: {}).",
                jwt_secret.len()
            ));
        }
    }

    // 2. S3 credentials must be provided as a complete set.
    //    We treat the bucket as the feature-flag: if it is set then the full
    //    set of credentials is required.
    let s3_fields = [
        ("S3_BUCKET", s3_bucket),
        ("S3_ACCESS_KEY_ID", s3_access_key_id),
        ("S3_SECRET_ACCESS_KEY", s3_secret_access_key),
        ("S3_PUBLIC_URL", s3_public_url),
    ];
    let s3_present: Vec<&str> = s3_fields
        .iter()
        .filter(|(_, v)| !v.is_empty())
        .map(|(k, _)| *k)
        .collect();
    let s3_missing: Vec<&str> = s3_fields
        .iter()
        .filter(|(_, v)| v.is_empty())
        .map(|(k, _)| *k)
        .collect();

    if !s3_present.is_empty() && !s3_missing.is_empty() {
        errors.push(format!(
            "Partial S3 configuration detected. \
             The following S3 variables are set: [{}] but these are missing: [{}]. \
             Either configure all S3 variables together or leave them all unset \
             to disable image uploads.",
            s3_present.join(", "),
            s3_missing.join(", ")
        ));
    }

    // 3. S3_PUBLIC_URL must be a valid URL when set.
    if !s3_public_url.is_empty() {
        if let Err(e) = parse_url_value(s3_public_url) {
            errors.push(format!("S3_PUBLIC_URL is not a valid URL: {e}"));
        }
    }

    // 4. S3_ENDPOINT_URL is required for Cloudflare R2 (region = "auto").
    //    Warn — not hard-error — because some S3-compatible providers don't
    //    need a custom endpoint.  We only enforce this when S3 is configured
    //    and region is "auto" (the R2 default).
    // (This is advisory; we do not push to errors to avoid breaking AWS S3 setups.)

    // 5. Production CORS origins must all be HTTPS.
    if is_production {
        for origin in cors_allowed_origins.split(',') {
            let trimmed = origin.trim();
            if trimmed.is_empty() {
                continue;
            }
            match Url::parse(trimmed) {
                Err(_) => {
                    errors.push(format!(
                        "CORS_ALLOWED_ORIGINS contains an invalid URL in production: {trimmed:?}"
                    ));
                }
                Ok(parsed) if parsed.scheme() != "https" => {
                    errors.push(format!(
                        "CORS_ALLOWED_ORIGINS contains a non-HTTPS origin in production: \
                         {trimmed:?}. All origins must use HTTPS in production."
                    ));
                }
                _ => {}
            }
        }
    } else {
        // In non-production, just validate that each entry is a parseable URL.
        for origin in cors_allowed_origins.split(',') {
            let trimmed = origin.trim();
            if trimmed.is_empty() {
                continue;
            }
            if let Err(e) = parse_url_value(trimmed) {
                errors.push(format!(
                    "CORS_ALLOWED_ORIGINS contains an invalid URL {trimmed:?}: {e}"
                ));
            }
        }
    }

    // 6. S3_ENDPOINT_URL present but bucket not set → likely misconfiguration.
    if s3_endpoint_url.is_some() && s3_bucket.is_empty() {
        errors.push(
            "S3_ENDPOINT_URL is set but S3_BUCKET is not — \
             a custom endpoint is only meaningful when a bucket is configured."
                .to_string(),
        );
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // Serialize all env-var tests to prevent cross-test interference.
    static ENV_MUTEX: Mutex<()> = Mutex::new(());

    /// Minimal set of env vars that produces a valid Config.
    fn set_valid_env() {
        env::set_var("DATABASE_URL", "postgres://user:pass@localhost:5432/agora");
        env::set_var("JWT_SECRET", "a-secret-that-is-long-enough-for-tests-ok");
        env::remove_var("PORT");
        env::remove_var("RUST_ENV");
        env::remove_var("RUST_LOG");
        env::remove_var("CORS_ALLOWED_ORIGINS");
        env::remove_var("SOROBAN_RPC_URL");
        env::remove_var("REDIS_URL");
        env::remove_var("BASE_URL");
        env::remove_var("S3_BUCKET");
        env::remove_var("S3_ACCESS_KEY_ID");
        env::remove_var("S3_SECRET_ACCESS_KEY");
        env::remove_var("S3_PUBLIC_URL");
        env::remove_var("S3_ENDPOINT_URL");
        env::remove_var("S3_REGION");
    }

    fn clear_env() {
        for var in &[
            "DATABASE_URL", "PORT", "RUST_ENV", "RUST_LOG",
            "CORS_ALLOWED_ORIGINS", "SOROBAN_RPC_URL", "REDIS_URL", "BASE_URL",
            "JWT_SECRET", "S3_BUCKET", "S3_ACCESS_KEY_ID", "S3_SECRET_ACCESS_KEY",
            "S3_PUBLIC_URL", "S3_ENDPOINT_URL", "S3_REGION",
        ] {
            env::remove_var(var);
        }
    }

    // ── Happy-path ───────────────────────────────────────────────────────────

    #[test]
    fn test_valid_config_loads() {
        let _g = ENV_MUTEX.lock().unwrap();
        set_valid_env();
        assert!(Config::from_env().is_ok());
        clear_env();
    }

    #[test]
    fn test_defaults_applied() {
        let _g = ENV_MUTEX.lock().unwrap();
        set_valid_env();
        let c = Config::from_env().unwrap();
        assert_eq!(c.port, 3001);
        assert_eq!(c.rust_env, "development");
        assert_eq!(c.rust_log, "info");
        assert_eq!(c.cors_allowed_origins, "http://localhost:3000,http://localhost:5173");
        assert_eq!(c.soroban_rpc_url, "https://soroban-testnet.stellar.org");
        assert_eq!(c.redis_url, "redis://127.0.0.1:6379");
        assert_eq!(c.base_url, "https://agora.events");
        clear_env();
    }

    #[test]
    fn test_custom_port() {
        let _g = ENV_MUTEX.lock().unwrap();
        set_valid_env();
        env::set_var("PORT", "8080");
        let c = Config::from_env().unwrap();
        assert_eq!(c.port, 8080);
        clear_env();
    }

    #[test]
    fn test_is_production_flag() {
        let _g = ENV_MUTEX.lock().unwrap();
        set_valid_env();
        env::set_var("RUST_ENV", "production");
        env::set_var("JWT_SECRET", "a-very-long-and-secure-secret-for-production-use");
        env::set_var("CORS_ALLOWED_ORIGINS", "https://agora.events");
        let c = Config::from_env().unwrap();
        assert!(c.is_production());
        clear_env();
    }

    #[test]
    fn test_s3_enabled_when_all_fields_set() {
        let _g = ENV_MUTEX.lock().unwrap();
        set_valid_env();
        env::set_var("S3_BUCKET", "mybucket");
        env::set_var("S3_ACCESS_KEY_ID", "keyid");
        env::set_var("S3_SECRET_ACCESS_KEY", "secretkey");
        env::set_var("S3_PUBLIC_URL", "https://cdn.example.com");
        let c = Config::from_env().unwrap();
        assert!(c.s3_enabled());
        clear_env();
    }

    #[test]
    fn test_s3_disabled_when_no_fields_set() {
        let _g = ENV_MUTEX.lock().unwrap();
        set_valid_env();
        let c = Config::from_env().unwrap();
        assert!(!c.s3_enabled());
        clear_env();
    }

    // ── Missing / invalid required vars ─────────────────────────────────────

    #[test]
    fn test_missing_database_url_fails() {
        let _g = ENV_MUTEX.lock().unwrap();
        set_valid_env();
        env::remove_var("DATABASE_URL");
        let err = Config::from_env().unwrap_err().to_string();
        assert!(err.contains("DATABASE_URL"), "expected DATABASE_URL in: {err}");
        clear_env();
    }

    #[test]
    fn test_empty_database_url_fails() {
        let _g = ENV_MUTEX.lock().unwrap();
        set_valid_env();
        env::set_var("DATABASE_URL", "");
        let err = Config::from_env().unwrap_err().to_string();
        assert!(err.contains("DATABASE_URL"), "expected DATABASE_URL in: {err}");
        clear_env();
    }

    #[test]
    fn test_malformed_database_url_fails() {
        let _g = ENV_MUTEX.lock().unwrap();
        set_valid_env();
        env::set_var("DATABASE_URL", "not a url at all");
        let err = Config::from_env().unwrap_err().to_string();
        assert!(err.contains("DATABASE_URL"), "expected DATABASE_URL in: {err}");
        clear_env();
    }

    // ── Port validation ──────────────────────────────────────────────────────

    #[test]
    fn test_port_zero_fails() {
        let _g = ENV_MUTEX.lock().unwrap();
        set_valid_env();
        env::set_var("PORT", "0");
        let err = Config::from_env().unwrap_err().to_string();
        assert!(err.contains("PORT"), "expected PORT in: {err}");
        clear_env();
    }

    #[test]
    fn test_port_out_of_range_fails() {
        let _g = ENV_MUTEX.lock().unwrap();
        set_valid_env();
        env::set_var("PORT", "99999");
        let err = Config::from_env().unwrap_err().to_string();
        assert!(err.contains("PORT"), "expected PORT in: {err}");
        clear_env();
    }

    #[test]
    fn test_port_non_numeric_fails() {
        let _g = ENV_MUTEX.lock().unwrap();
        set_valid_env();
        env::set_var("PORT", "abc");
        let err = Config::from_env().unwrap_err().to_string();
        assert!(err.contains("PORT"), "expected PORT in: {err}");
        clear_env();
    }

    #[test]
    fn test_port_boundary_values() {
        let _g = ENV_MUTEX.lock().unwrap();
        for port in [1u16, 80, 443, 8080, 65535] {
            set_valid_env();
            env::set_var("PORT", port.to_string());
            let c = Config::from_env().expect(&format!("port {port} should be valid"));
            assert_eq!(c.port, port);
        }
        clear_env();
    }

    // ── RUST_ENV validation ──────────────────────────────────────────────────

    #[test]
    fn test_invalid_rust_env_fails() {
        let _g = ENV_MUTEX.lock().unwrap();
        set_valid_env();
        env::set_var("RUST_ENV", "superproduction");
        let err = Config::from_env().unwrap_err().to_string();
        assert!(err.contains("RUST_ENV"), "expected RUST_ENV in: {err}");
        clear_env();
    }

    #[test]
    fn test_valid_rust_env_values() {
        let _g = ENV_MUTEX.lock().unwrap();
        for env_val in &["development", "production", "testing", "staging"] {
            set_valid_env();
            env::set_var("RUST_ENV", env_val);
            if *env_val == "production" {
                env::set_var("JWT_SECRET", "a-very-long-and-secure-secret-for-production-use");
                env::set_var("CORS_ALLOWED_ORIGINS", "https://agora.events");
            }
            Config::from_env().expect(&format!("RUST_ENV={env_val} should be valid"));
        }
        clear_env();
    }

    // ── RUST_LOG validation ──────────────────────────────────────────────────

    #[test]
    fn test_invalid_rust_log_fails() {
        let _g = ENV_MUTEX.lock().unwrap();
        set_valid_env();
        env::set_var("RUST_LOG", "verbose");
        let err = Config::from_env().unwrap_err().to_string();
        assert!(err.contains("RUST_LOG"), "expected RUST_LOG in: {err}");
        clear_env();
    }

    #[test]
    fn test_valid_rust_log_levels() {
        let _g = ENV_MUTEX.lock().unwrap();
        for level in &["trace", "debug", "info", "warn", "error", "off"] {
            set_valid_env();
            env::set_var("RUST_LOG", level);
            Config::from_env().expect(&format!("RUST_LOG={level} should be valid"));
        }
        clear_env();
    }

    #[test]
    fn test_crate_specific_rust_log_accepted() {
        let _g = ENV_MUTEX.lock().unwrap();
        set_valid_env();
        env::set_var("RUST_LOG", "agora_server=debug,info");
        Config::from_env().expect("crate-specific RUST_LOG should be accepted");
        clear_env();
    }

    // ── URL validation ───────────────────────────────────────────────────────

    #[test]
    fn test_malformed_soroban_rpc_url_fails() {
        let _g = ENV_MUTEX.lock().unwrap();
        set_valid_env();
        env::set_var("SOROBAN_RPC_URL", "not-a-url");
        let err = Config::from_env().unwrap_err().to_string();
        assert!(err.contains("SOROBAN_RPC_URL"), "expected SOROBAN_RPC_URL in: {err}");
        clear_env();
    }

    #[test]
    fn test_malformed_redis_url_fails() {
        let _g = ENV_MUTEX.lock().unwrap();
        set_valid_env();
        env::set_var("REDIS_URL", "://broken");
        let err = Config::from_env().unwrap_err().to_string();
        assert!(err.contains("REDIS_URL"), "expected REDIS_URL in: {err}");
        clear_env();
    }

    #[test]
    fn test_malformed_base_url_fails() {
        let _g = ENV_MUTEX.lock().unwrap();
        set_valid_env();
        env::set_var("BASE_URL", "htp//bad");
        let err = Config::from_env().unwrap_err().to_string();
        assert!(err.contains("BASE_URL"), "expected BASE_URL in: {err}");
        clear_env();
    }

    #[test]
    fn test_malformed_s3_endpoint_url_fails() {
        let _g = ENV_MUTEX.lock().unwrap();
        set_valid_env();
        env::set_var("S3_ENDPOINT_URL", "not_a_url");
        env::set_var("S3_BUCKET", "mybucket");
        env::set_var("S3_ACCESS_KEY_ID", "key");
        env::set_var("S3_SECRET_ACCESS_KEY", "secret");
        env::set_var("S3_PUBLIC_URL", "https://cdn.example.com");
        let err = Config::from_env().unwrap_err().to_string();
        assert!(err.contains("S3_ENDPOINT_URL"), "expected S3_ENDPOINT_URL in: {err}");
        clear_env();
    }

    // ── CORS validation ──────────────────────────────────────────────────────

    #[test]
    fn test_invalid_cors_origin_fails() {
        let _g = ENV_MUTEX.lock().unwrap();
        set_valid_env();
        env::set_var("CORS_ALLOWED_ORIGINS", "not-a-url,http://localhost:3000");
        let err = Config::from_env().unwrap_err().to_string();
        assert!(err.contains("CORS_ALLOWED_ORIGINS"), "expected CORS_ALLOWED_ORIGINS in: {err}");
        clear_env();
    }

    #[test]
    fn test_production_http_cors_origin_fails() {
        let _g = ENV_MUTEX.lock().unwrap();
        set_valid_env();
        env::set_var("RUST_ENV", "production");
        env::set_var("JWT_SECRET", "a-very-long-and-secure-secret-for-production-use");
        env::set_var("CORS_ALLOWED_ORIGINS", "http://insecure.example.com");
        let err = Config::from_env().unwrap_err().to_string();
        assert!(err.contains("CORS_ALLOWED_ORIGINS"), "expected CORS_ALLOWED_ORIGINS in: {err}");
        assert!(err.contains("HTTPS") || err.contains("https"), "expected HTTPS mention in: {err}");
        clear_env();
    }

    // ── JWT_SECRET / production incompatibilities ────────────────────────────

    #[test]
    fn test_production_with_fallback_jwt_secret_fails() {
        let _g = ENV_MUTEX.lock().unwrap();
        set_valid_env();
        env::set_var("RUST_ENV", "production");
        env::set_var("CORS_ALLOWED_ORIGINS", "https://agora.events");
        env::remove_var("JWT_SECRET"); // triggers fallback
        let err = Config::from_env().unwrap_err().to_string();
        assert!(err.contains("JWT_SECRET"), "expected JWT_SECRET in: {err}");
        clear_env();
    }

    #[test]
    fn test_production_with_short_jwt_secret_fails() {
        let _g = ENV_MUTEX.lock().unwrap();
        set_valid_env();
        env::set_var("RUST_ENV", "production");
        env::set_var("CORS_ALLOWED_ORIGINS", "https://agora.events");
        env::set_var("JWT_SECRET", "tooshort");
        let err = Config::from_env().unwrap_err().to_string();
        assert!(err.contains("JWT_SECRET"), "expected JWT_SECRET in: {err}");
        clear_env();
    }

    #[test]
    fn test_development_allows_weak_jwt_secret() {
        let _g = ENV_MUTEX.lock().unwrap();
        set_valid_env();
        env::set_var("RUST_ENV", "development");
        env::remove_var("JWT_SECRET"); // fallback is fine in dev
        assert!(Config::from_env().is_ok());
        clear_env();
    }

    // ── S3 partial configuration ─────────────────────────────────────────────

    #[test]
    fn test_partial_s3_config_fails() {
        let _g = ENV_MUTEX.lock().unwrap();
        set_valid_env();
        env::set_var("S3_BUCKET", "mybucket");
        // deliberately omit S3_ACCESS_KEY_ID, S3_SECRET_ACCESS_KEY, S3_PUBLIC_URL
        let err = Config::from_env().unwrap_err().to_string();
        assert!(err.contains("S3"), "expected S3 mention in: {err}");
        clear_env();
    }

    #[test]
    fn test_s3_endpoint_without_bucket_fails() {
        let _g = ENV_MUTEX.lock().unwrap();
        set_valid_env();
        env::set_var("S3_ENDPOINT_URL", "https://r2.cloudflarestorage.com");
        // no S3_BUCKET
        let err = Config::from_env().unwrap_err().to_string();
        assert!(err.contains("S3_ENDPOINT_URL") || err.contains("S3_BUCKET"),
            "expected S3 mention in: {err}");
        clear_env();
    }

    // ── Multiple errors reported together ────────────────────────────────────

    #[test]
    fn test_multiple_errors_reported_together() {
        let _g = ENV_MUTEX.lock().unwrap();
        clear_env();
        env::set_var("DATABASE_URL", "bad url");
        env::set_var("PORT", "0");
        env::set_var("RUST_ENV", "unknownenv");
        let err = Config::from_env().unwrap_err().to_string();
        // All three problems must appear in a single error message.
        assert!(err.contains("DATABASE_URL"), "missing DATABASE_URL in: {err}");
        assert!(err.contains("PORT"), "missing PORT in: {err}");
        assert!(err.contains("RUST_ENV"), "missing RUST_ENV in: {err}");
        clear_env();
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
}
