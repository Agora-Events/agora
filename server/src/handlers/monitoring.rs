//! # Monitoring Handlers
//!
//! Exposes internal operational metrics (DB pool utilization, etc.) for
//! observability tooling. Protected by a static API key so that unauthenticated
//! callers can't use it for infrastructure reconnaissance.

use axum::{
    extract::State,
    http::{header::AUTHORIZATION, HeaderMap},
    response::{IntoResponse, Response},
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use sqlx::PgPool;

use crate::utils::error::AppError;
use crate::utils::response::success;

#[derive(Debug, Serialize)]
pub struct MonitoringMetrics {
    pub db_pool_size: u32,
    pub db_pool_idle: usize,
    pub db_pool_in_use: usize,
}

/// Validates the `Authorization: Bearer <key>` header against the
/// `MONITORING_API_KEY` environment variable.
fn is_authorized(headers: &HeaderMap) -> bool {
    let expected = match std::env::var("MONITORING_API_KEY") {
        Ok(key) if !key.is_empty() => key,
        _ => return false,
    };

    let provided = headers
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));

    match provided {
        Some(token) => {
            // Compare fixed-size SHA-256 digests with an XOR accumulator so the
            // number of instructions does not depend on where the tokens differ.
            let expected_digest = Sha256::digest(expected.as_bytes());
            let provided_digest = Sha256::digest(token.as_bytes());
            let mut difference = 0u8;
            for (expected_byte, provided_byte) in expected_digest.iter().zip(provided_digest.iter())
            {
                difference |= expected_byte ^ provided_byte;
            }
            difference == 0
        }
        None => false,
    }
}

/// Get internal monitoring metrics.
///
/// # Endpoint
/// GET `/api/v1/monitoring`
///
/// Requires an `Authorization: Bearer <MONITORING_API_KEY>` header. Requests
/// without a valid credential are rejected with 401.
pub async fn get_monitoring(State(pool): State<PgPool>, headers: HeaderMap) -> Response {
    if !is_authorized(&headers) {
        return AppError::AuthError("A valid monitoring credential is required".to_string())
            .into_response();
    }

    let db_pool_size = pool.size();
    let db_pool_idle = pool.num_idle();
    let metrics = MonitoringMetrics {
        db_pool_size,
        db_pool_idle,
        db_pool_in_use: (db_pool_size as usize).saturating_sub(db_pool_idle),
    };

    success(metrics, "Monitoring metrics retrieved successfully").into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{HeaderValue, Request, StatusCode},
        Router,
    };
    use std::sync::Mutex;
    use tower::ServiceExt;

    // MONITORING_API_KEY is process-global env state; serialize these tests.
    static ENV_MUTEX: Mutex<()> = Mutex::new(());

    fn test_pool() -> PgPool {
        sqlx::postgres::PgPoolOptions::new()
            .connect_lazy("postgres://agora:agora@127.0.0.1:1/agora")
            .expect("lazy pool should be constructible")
    }

    async fn monitoring_status(bearer_key: Option<&str>) -> StatusCode {
        let mut builder = Request::builder().uri("/monitoring");
        if let Some(key) = bearer_key {
            builder = builder.header(
                AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {key}")).unwrap(),
            );
        }
        let router = Router::new()
            .route("/monitoring", get(get_monitoring))
            .with_state(test_pool());
        router
            .oneshot(builder.body(Body::empty()).unwrap())
            .await
            .unwrap()
            .status()
    }

    #[tokio::test]
    async fn test_missing_key_returns_401() {
        let _guard = ENV_MUTEX.lock().unwrap();
        std::env::set_var("MONITORING_API_KEY", "secret-key");
        assert_eq!(monitoring_status(None).await, StatusCode::UNAUTHORIZED);
        std::env::remove_var("MONITORING_API_KEY");
    }

    #[tokio::test]
    async fn test_wrong_key_returns_401() {
        let _guard = ENV_MUTEX.lock().unwrap();
        std::env::set_var("MONITORING_API_KEY", "secret-key");
        assert_eq!(
            monitoring_status(Some("wrong-key")).await,
            StatusCode::UNAUTHORIZED
        );
        std::env::remove_var("MONITORING_API_KEY");
    }

    #[tokio::test]
    async fn test_correct_key_returns_200() {
        let _guard = ENV_MUTEX.lock().unwrap();
        std::env::set_var("MONITORING_API_KEY", "secret-key");
        assert_eq!(monitoring_status(Some("secret-key")).await, StatusCode::OK);
        std::env::remove_var("MONITORING_API_KEY");
    }

    #[tokio::test]
    async fn test_unset_key_returns_401() {
        let _guard = ENV_MUTEX.lock().unwrap();
        std::env::remove_var("MONITORING_API_KEY");
        assert_eq!(
            monitoring_status(Some("secret-key")).await,
            StatusCode::UNAUTHORIZED
        );
    }

    #[test]
    fn test_rejects_missing_credential() {
        let _guard = ENV_MUTEX.lock().unwrap();
        std::env::set_var("MONITORING_API_KEY", "secret-key");
        let headers = HeaderMap::new();
        assert!(!is_authorized(&headers));
        std::env::remove_var("MONITORING_API_KEY");
    }

    #[test]
    fn test_rejects_wrong_credential() {
        let _guard = ENV_MUTEX.lock().unwrap();
        std::env::set_var("MONITORING_API_KEY", "secret-key");
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, HeaderValue::from_static("Bearer wrong-key"));
        assert!(!is_authorized(&headers));
        std::env::remove_var("MONITORING_API_KEY");
    }

    #[test]
    fn test_accepts_valid_credential() {
        let _guard = ENV_MUTEX.lock().unwrap();
        std::env::set_var("MONITORING_API_KEY", "secret-key");
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, HeaderValue::from_static("Bearer secret-key"));
        assert!(is_authorized(&headers));
        std::env::remove_var("MONITORING_API_KEY");
    }

    #[test]
    fn test_rejects_when_key_not_configured() {
        let _guard = ENV_MUTEX.lock().unwrap();
        std::env::remove_var("MONITORING_API_KEY");
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, HeaderValue::from_static("Bearer anything"));
        assert!(!is_authorized(&headers));
    }
}
