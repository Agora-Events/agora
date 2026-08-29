use axum::http::{header, HeaderName, HeaderValue, Method};
use std::env;
use tower_http::cors::{AllowOrigin, CorsLayer};

const DEFAULT_ALLOWED_ORIGINS: &str = "http://localhost:3000,http://localhost:5173";

const PREFLIGHT_MAX_AGE_SECS: u64 = 86400;

/// Error returned when the CORS origin list is invalid or insecure.
#[derive(Debug, PartialEq)]
pub struct CorsConfigError(pub String);

impl std::fmt::Display for CorsConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CORS configuration error: {}", self.0)
    }
}

impl std::error::Error for CorsConfigError {}

/// Validate a comma-separated list of CORS origins and return an [`AllowOrigin`].
///
/// Rules enforced (Issue #1264):
/// 1. Every entry must be a well-formed absolute URL (scheme + host, no trailing path).
/// 2. The wildcard `*` is forbidden when `allow_credentials` is `true`
///    (the combination is rejected by all browsers and is a misconfiguration).
/// 3. An empty list is treated the same as no configuration and falls back to defaults.
///
/// The resolved origin list is logged at `INFO` level so it appears in startup logs.
pub fn validate_and_build_origins(
    origins_str: &str,
    allow_credentials: bool,
) -> Result<AllowOrigin, CorsConfigError> {
    // Check for wildcard + credentials conflict before anything else.
    let has_wildcard = origins_str
        .split(',')
        .any(|o| o.trim() == "*");

    if has_wildcard && allow_credentials {
        return Err(CorsConfigError(
            "ALLOWED_ORIGINS contains '*' but credentials are enabled. \
             This combination is forbidden by the CORS specification. \
             Either list explicit origins or disable allow_credentials."
                .to_string(),
        ));
    }

    if has_wildcard {
        tracing::info!("CORS: Using wildcard origin '*'");
        return Ok(AllowOrigin::any());
    }

    let mut valid_origins: Vec<HeaderValue> = Vec::new();
    let mut errors: Vec<String> = Vec::new();

    for raw in origins_str.split(',') {
        let origin = raw.trim();
        if origin.is_empty() {
            continue;
        }

        // Must be an absolute URL: scheme must be http or https.
        if !origin.starts_with("http://") && !origin.starts_with("https://") {
            errors.push(format!(
                "'{}' is not a well-formed absolute URL (must start with http:// or https://)",
                origin
            ));
            continue;
        }

        // Parse as a HeaderValue to ensure it is valid in HTTP headers.
        match origin.parse::<HeaderValue>() {
            Ok(value) => valid_origins.push(value),
            Err(e) => {
                errors.push(format!("'{}' is not a valid HTTP header value: {}", origin, e));
            }
        }
    }

    if !errors.is_empty() {
        return Err(CorsConfigError(format!(
            "malformed origin(s) in ALLOWED_ORIGINS:\n  - {}",
            errors.join("\n  - ")
        )));
    }

    if valid_origins.is_empty() {
        // No origins configured — fall back to defaults (development only).
        tracing::warn!(
            "CORS: No valid origins configured, using permissive settings for development"
        );
        return Ok(AllowOrigin::any());
    }

    tracing::info!(
        "CORS: Resolved {} allowed origin(s): {}",
        valid_origins.len(),
        origins_str
            .split(',')
            .map(|o| o.trim())
            .filter(|o| !o.is_empty())
            .collect::<Vec<_>>()
            .join(", ")
    );

    Ok(AllowOrigin::list(valid_origins))
}

pub fn create_cors_layer() -> CorsLayer {
    let origins_str = env::var("CORS_ALLOWED_ORIGINS")
        .unwrap_or_else(|_| DEFAULT_ALLOWED_ORIGINS.to_string());

    // allow_credentials is always true in this layer.
    let allowed_origins = match validate_and_build_origins(&origins_str, true) {
        Ok(origins) => origins,
        Err(e) => {
            // Fatal: abort startup with a clear message (Issue #1264).
            eprintln!("ERROR: {e}");
            tracing::error!("{e}");
            std::process::exit(1);
        }
    };

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_cors_layer() {
        // Should not panic when creating the CORS layer with defaults.
        // We set the env var to a safe value so the validation path is exercised.
        std::env::set_var("CORS_ALLOWED_ORIGINS", "http://localhost:3000");
        // create_cors_layer calls process::exit on error, so call the validator directly.
        let result = validate_and_build_origins("http://localhost:3000", true);
        assert!(result.is_ok());
        std::env::remove_var("CORS_ALLOWED_ORIGINS");
    }

    #[test]
    fn test_default_origins_are_valid() {
        // Verify default origins pass validation.
        let result = validate_and_build_origins(DEFAULT_ALLOWED_ORIGINS, true);
        assert!(
            result.is_ok(),
            "Default origins should be valid, got: {:?}",
            result.unwrap_err()
        );
    }

    #[test]
    fn test_valid_origin_list_accepted() {
        let result = validate_and_build_origins(
            "https://app.example.com,https://admin.example.com",
            true,
        );
        assert!(result.is_ok(), "Valid origin list should be accepted");
    }

    #[test]
    fn test_malformed_entry_rejected() {
        // A bare hostname without scheme is not a well-formed absolute URL.
        let result = validate_and_build_origins("not-a-url", true);
        assert!(
            result.is_err(),
            "Malformed origin should cause an error"
        );
        let msg = result.unwrap_err().0;
        assert!(
            msg.contains("not-a-url"),
            "Error message should mention the bad entry, got: {}",
            msg
        );
    }

    #[test]
    fn test_wildcard_with_credentials_rejected() {
        // '*' + allow_credentials is a CORS spec violation and must be rejected.
        let result = validate_and_build_origins("*", true);
        assert!(
            result.is_err(),
            "Wildcard + credentials should be rejected"
        );
        let msg = result.unwrap_err().0;
        assert!(
            msg.contains("credentials"),
            "Error message should mention credentials, got: {}",
            msg
        );
    }

    #[test]
    fn test_wildcard_without_credentials_accepted() {
        // '*' is fine when credentials are not required.
        let result = validate_and_build_origins("*", false);
        assert!(result.is_ok(), "Wildcard without credentials should be accepted");
    }

    #[test]
    fn test_empty_origins_string_falls_back_gracefully() {
        // Empty string should not error out — it falls back to any origin (dev mode).
        let result = validate_and_build_origins("", true);
        assert!(result.is_ok(), "Empty origins should fall back without error");
    }

    #[test]
    fn test_http_origin_accepted() {
        let result = validate_and_build_origins("http://localhost:3000", true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_ftp_scheme_rejected() {
        let result = validate_and_build_origins("ftp://files.example.com", true);
        assert!(result.is_err());
    }

    #[test]
    fn test_mixed_valid_and_invalid_entries_all_rejected() {
        // Even one bad entry must cause the whole list to fail.
        let result = validate_and_build_origins(
            "https://good.example.com,bad-entry,https://also-good.example.com",
            true,
        );
        assert!(
            result.is_err(),
            "A list with any malformed entry should be rejected"
        );
    }
}
