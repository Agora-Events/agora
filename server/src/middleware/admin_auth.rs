//! # Admin Auth Middleware
//!
//! Protects administrative API routes with a static bearer token configured via
//! the `ADMIN_TOKEN` environment variable.
//!
//! If the token is absent or invalid, requests are rejected with HTTP 401.

use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

/// Application state passed to this middleware.
#[derive(Clone)]
pub struct AdminAuthState {
    pub token: Option<String>,
}

/// Axum middleware that enforces the admin bearer token.
pub async fn require_admin_token(
    State(state): State<AdminAuthState>,
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
                        "message": "Admin token is not configured"
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
                    "message": "Missing or invalid admin token"
                }
            })),
        )
            .into_response(),
    }
}

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
        let state = AdminAuthState {
            token: token.map(str::to_string),
        };
        Router::new()
            .route("/admin", get(|| async { "ok" }))
            .route_layer(middleware::from_fn_with_state(state, require_admin_token))
    }

    async fn call(router: Router, auth: Option<&str>) -> StatusCode {
        let mut builder = Request::builder().uri("/admin");
        if let Some(a) = auth {
            builder = builder.header("Authorization", a);
        }
        let req = builder.body(Body::empty()).unwrap();
        router.oneshot(req).await.unwrap().status()
    }

    #[tokio::test]
    async fn test_no_admin_token_configured_returns_401() {
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
        assert_eq!(call(router, Some("Bearer wrong")).await, StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_correct_token_passes_through() {
        let router = make_router(Some("secret"));
        assert_eq!(call(router, Some("Bearer secret")).await, StatusCode::OK);
    }
}
