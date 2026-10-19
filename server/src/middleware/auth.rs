//! # JWT Authentication Middleware (Issue #484)
//!
//! Protects authenticated routes by validating a Bearer JWT issued during
//! the Stellar wallet challenge-response flow and injecting the resolved
//! user identity as an [`AuthUser`] extension.

use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::Deserialize;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use crate::utils::error::AppError;

/// JWT claims payload (mirrors [`crate::handlers::auth::Claims`]).
#[derive(Debug, serde::Deserialize)]
struct Claims {
    pub sub: String,
    pub iat: i64,
    pub exp: i64,
}

/// Authenticated user identity injected into protected handlers.
#[derive(Debug, Clone)]
pub struct AuthUser {
    /// Internal database UUID of the authenticated user.
    pub user_id: Uuid,
    /// Stellar wallet address (G… or C… format) that authenticated.
    pub wallet: String,
}

fn extract_auth(headers: &axum::http::HeaderMap) -> Result<String, AppError> {
    let header = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| AppError::AuthError("Missing Authorization header".to_string()))?;

    let token = header.strip_prefix("Bearer ").ok_or_else(|| {
        AppError::AuthError("Authorization header must use Bearer scheme".to_string())
    })?;

    let dev_prefix = format!("{}{}", "sk_", "live_");
    if token.starts_with(&dev_prefix) {
        return Err(AppError::AuthError("Invalid API key".to_string()));
    }

    // Inline JWT verification to avoid a circular dependency on handlers::auth.
    let jwt_secret = std::env::var("JWT_SECRET").map_err(|_| {
        AppError::AuthError("JWT secret not configured".to_string())
    })?;

    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;
    let claims = decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret.as_bytes()),
        &validation,
    )
    .map(|data| data.claims)
    .map_err(|e| AppError::AuthError(format!("Invalid or expired token: {e}")))?;

    Ok(claims.sub)
}

/// Axum middleware that enforces a valid JWT and injects [`AuthUser`].
pub async fn require_auth(
    State(pool): State<PgPool>,
    request: Request,
    next: Next,
) -> Response {
    let wallet = match extract_auth(request.headers()) {
        Ok(w) => w,
        Err(_) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "success": false,
                    "error": {
                        "code": "UNAUTHORIZED",
                        "message": "Missing or invalid authorization token"
                    }
                })),
            )
                .into_response();
        }
    };

    // Resolve the wallet address to an internal user UUID.
    let user_id: Option<Uuid> = sqlx::query_scalar("SELECT id FROM users WHERE wallet_address = $1")
        .bind(&wallet)
        .fetch_optional(&pool)
        .await
        .ok()
        .flatten();

    let user_id = match user_id {
        Some(id) => id,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "success": false,
                    "error": {
                        "code": "UNAUTHORIZED",
                        "message": "User not found"
                    }
                })),
            )
                .into_response();
        }
    };

    let auth_user = AuthUser { user_id, wallet };
    let mut req = request;
    req.extensions_mut().insert(auth_user);
    next.run(req).await
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

    // Helper to build a router with the auth middleware.
    fn make_router() -> Router {
        Router::new()
            .route("/protected", get(|| async { "authenticated" }))
            .route_layer(middleware::from_fn_with_state(
                PgPool::connect_lazy("postgresql://localhost/test").expect("test pool"),
                require_auth,
            ))
    }

    async fn call(router: Router, auth_header: Option<&str>) -> (StatusCode, String) {
        let mut builder = Request::builder().uri("/protected");
        if let Some(h) = auth_header {
            builder = builder.header("Authorization", h);
        }
        let req = builder.body(Body::empty()).unwrap();
        let resp = router.oneshot(req).await.unwrap();
        let status = resp.status();
        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let text = String::from_utf8_lossy(&body).into_owned();
        (status, text)
    }

    #[tokio::test]
    async fn test_missing_token_returns_401() {
        let (status, _) = call(make_router(), None).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_invalid_token_returns_401() {
        let (status, body) = call(make_router(), Some("Bearer invalid.token.here")).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
        assert!(body.contains("UNAUTHORIZED"));
    }

    #[tokio::test]
    async fn test_malformed_header_returns_401() {
        let (status, _) = call(make_router(), Some("NotBearer token")).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
    }
}
