//! CORS (Cross-Origin Resource Sharing) configuration for the Agora API.
//!
//! This module provides CORS middleware configuration that allows the backend
//! to safely communicate with frontend applications while maintaining security.

use axum::http::{header, HeaderName, HeaderValue, Method};
use std::env;
use tower_http::cors::{AllowOrigin, CorsLayer};

/// Default allowed origins for development
const DEFAULT_ALLOWED_ORIGINS: &str = "http://localhost:3000,http://localhost:5173";

/// Maximum age for preflight request caching (24 hours in seconds)
const PREFLIGHT_MAX_AGE_SECS: u64 = 86400;

/// Creates a configured CORS layer for the Axum application.
///
/// # Configuration
///
/// The CORS layer is configured using environment variables:
/// - `CORS_ALLOWED_ORIGINS`: Comma-separated list of allowed origins
///   (e.g., "http://localhost:3000,https://agora.example.com")
///
/// # Features
///
/// - Supports credentials (cookies, authorization headers)
/// - Allows standard HTTP methods (GET, POST, PUT, DELETE, PATCH, OPTIONS)
/// - Allows required headers (Content-Type, Authorization, Accept)
/// - Exposes common response headers
/// - Sets preflight request max age to 24 hours
///
/// # Example
///
/// ```rust,ignore
/// use agora_server::config::cors::create_cors_layer;
///
/// let app = Router::new()
///     .route("/api", get(handler))
///     .layer(create_cors_layer());
/// ```
pub fn create_cors_layer() -> CorsLayer {
    let allowed_origins = get_allowed_origins();

    CorsLayer::new()
        .allow_origin(allowed_origins)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::PATCH,
            Method::OPTIONS,
        ])
        .allow_headers([
            header::CONTENT_TYPE,
            header::AUTHORIZATION,
            header::ACCEPT,
            header::ORIGIN,
            HeaderName::from_static("x-requested-with"),
        ])
        .expose_headers([
            header::CONTENT_LENGTH,
            header::CONTENT_TYPE,
            HeaderName::from_static("x-request-id"),
        ])
        .allow_credentials(true)
        .max_age(std::time::Duration::from_secs(PREFLIGHT_MAX_AGE_SECS))
}

/// Retrieves and parses allowed origins from environment variables.
///
/// Reads the `CORS_ALLOWED_ORIGINS` environment variable and parses it
/// as a comma-separated list of origins. Falls back to default development
/// origins if the variable is not set.
fn get_allowed_origins() -> AllowOrigin {
    let origins_str =
        env::var("CORS_ALLOWED_ORIGINS").unwrap_or_else(|_| DEFAULT_ALLOWED_ORIGINS.to_string());

    let origins: Vec<HeaderValue> = origins_str
        .split(',')
        .filter_map(|origin| {
            let trimmed = origin.trim();
            if trimmed.is_empty() {
                None
            } else {
                match trimmed.parse::<HeaderValue>() {
                    Ok(value) => {
                        tracing::debug!("CORS: Allowing origin: {}", trimmed);
                        Some(value)
                    }
                    Err(e) => {
                        tracing::warn!("CORS: Invalid origin '{}': {}", trimmed, e);
                        None
                    }
                }
            }
        })
        .collect();

    if origins.is_empty() {
        tracing::warn!(
            "CORS: No valid origins configured, using permissive settings for development"
        );
        AllowOrigin::any()
    } else {
        tracing::info!("CORS: Configured with {} allowed origin(s)", origins.len());
        AllowOrigin::list(origins)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_cors_layer() {
        // Should not panic when creating the CORS layer
        let _layer = create_cors_layer();
    }

    #[test]
    fn test_default_origins_are_valid() {
        // Verify default origins can be parsed as HeaderValues
        for origin in DEFAULT_ALLOWED_ORIGINS.split(',') {
            let trimmed = origin.trim();
            assert!(
                trimmed.parse::<HeaderValue>().is_ok(),
                "Default origin '{}' should be a valid HeaderValue",
                trimmed
            );
        }
    }
}
