//! # Monitoring Auth Middleware
//!
//! Protects the `GET /api/v1/monitoring/dashboard` endpoint with a static
//! bearer token configured via the `MONITORING_TOKEN` environment variable.
//!
//! ## Behaviour
//! - If `MONITORING_TOKEN` is **not set**, all requests are rejected (fail-safe).
//! - If the `Authorization: Bearer <token>` header is missing or the token
//!   does not match, the middleware returns HTTP 401.
//! - On a valid token the request is forwarded to the inner handler unchanged.

use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

/// Application state passed to this middleware.
///
/// Only `monitoring_token` is needed, but we accept the full [`String`]
/// directly so the middleware can be wired with `from_fn_with_state`.
#[derive(Clone)]
pub struct MonitoringAuthState {
    /// The expected bearer token. `None` means no token is configured and
    /// every request will be rejected.
    pub token: Option<String>,
}

/// Axum middleware that enforces the monitoring bearer token.
///
/// Attach via `axum::middleware::from_fn_with_state` on the monitoring route.
pub async fn require_monitoring_token(
    State(state): State<MonitoringAuthState>,
    request: Request,
    next: Next,
) -> Response {
    let expected = match &state.token {
        Some(t) => t.clone(),
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "success": false,
                    "error": {
                        "code": "AUTH_ERROR",
                        "message": "Monitoring token is not configured"
                    }
                })),
            )
                .into_response();
        }
    };

    let provided = request
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(str::to_string);

    match provided {
        Some(token) if token == expected => next.run(request).await,
        _ => (
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "success": false,
                "error": {
                    "code": "AUTH_ERROR",
                    "message": "Missing or invalid monitoring token"
                }
            })),
        )
            .into_response(),
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        middleware,
        routing::get,
        Router,
    };
    use tower::ServiceExt;

    fn make_router(token: Option<&str>) -> Router {
        let state = MonitoringAuthState {
            token: token.map(str::to_string),
        };
        Router::new()
            .route("/dashboard", get(|| async { "ok" }))
            .route_layer(middleware::from_fn_with_state(
                state,
                require_monitoring_token,
            ))
    }

    async fn call(router: Router, auth: Option<&str>) -> StatusCode {
        let mut builder = Request::builder().uri("/dashboard");
        if let Some(a) = auth {
            builder = builder.header("Authorization", a);
        }
        let req = builder.body(Body::empty()).unwrap();
        router.oneshot(req).await.unwrap().status()
    }

    #[tokio::test]
    async fn test_no_token_configured_returns_401() {
        let router = make_router(None);
        assert_eq!(call(router, None).await, StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_missing_header_returns_401() {
        let router = make_router(Some("secret"));
        assert_eq!(call(router, None).await, StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_wrong_token_returns_401() {
        let router = make_router(Some("secret"));
        assert_eq!(
            call(router, Some("Bearer wrong")).await,
            StatusCode::UNAUTHORIZED
        );
    }

    #[tokio::test]
    async fn test_correct_token_passes_through() {
        let router = make_router(Some("secret"));
        assert_eq!(call(router, Some("Bearer secret")).await, StatusCode::OK);
    }

    #[tokio::test]
    async fn test_non_bearer_scheme_returns_401() {
        let router = make_router(Some("secret"));
        assert_eq!(
            call(router, Some("Basic secret")).await,
            StatusCode::UNAUTHORIZED
        );
    }
}
