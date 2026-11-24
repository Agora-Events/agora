use crate::utils::error::AppError;
use axum::{extract::Request, middleware::Next, response::Response};
use axum_extra::extract::cookie::CookieJar;

pub async fn check_csrf(req: Request, next: Next) -> Result<Response, AppError> {
    let jar = CookieJar::from_headers(req.headers());
    let method = req.method();
    if method == axum::http::Method::GET
        || method == axum::http::Method::HEAD
        || method == axum::http::Method::OPTIONS
    {
        return Ok(next.run(req).await);
    }

    let path = req.uri().path();
    if path.contains("/auth/nonce") || path.contains("/auth/verify") {
        return Ok(next.run(req).await);
    }

    let cookie_token = jar.get("XSRF-TOKEN").map(|c| c.value().to_string());
    let header_token = req
        .headers()
        .get("X-XSRF-TOKEN")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());

    if let (Some(cookie), Some(header)) = (cookie_token, header_token) {
        if cookie == header && !cookie.is_empty() {
            return Ok(next.run(req).await);
        }
    }

    Err(AppError::AuthError("CSRF validation failed".to_string()))
}

// ---------------------------------------------------------------------------
// Tests (Issue #1433)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Method, Request, StatusCode},
        middleware,
        routing::post,
        Router,
    };
    use tower::ServiceExt;

    fn make_router() -> Router {
        Router::new()
            .route("/api/v1/some-action", post(|| async { "ok" }))
            .route("/api/v1/auth/nonce", post(|| async { "ok" }))
            .layer(middleware::from_fn(check_csrf))
    }

    async fn call(
        router: Router,
        method: axum::http::Method,
        path: &str,
        cookie: Option<&str>,
        header: Option<&str>,
    ) -> StatusCode {
        let mut builder = Request::builder().uri(path).method(method);
        if let Some(c) = cookie {
            builder = builder.header("Cookie", format!("XSRF-TOKEN={}", c));
        }
        if let Some(h) = header {
            builder = builder.header("X-XSRF-TOKEN", h);
        }
        let req = builder.body(Body::empty()).unwrap();
        router.oneshot(req).await.unwrap().status()
    }

    #[tokio::test]
    async fn test_get_passes_without_token() {
        let router = make_router();
        let status = call(router, axum::http::Method::GET, "/api/v1/some-action", None, None).await;
        assert_eq!(status, StatusCode::OK);
    }

    #[tokio::test]
    async fn test_head_passes_without_token() {
        let router = make_router();
        let status = call(router, axum::http::Method::HEAD, "/api/v1/some-action", None, None).await;
        assert_eq!(status, StatusCode::OK);
    }

    #[tokio::test]
    async fn test_options_passes_without_token() {
        let router = make_router();
        let status = call(router, axum::http::Method::OPTIONS, "/api/v1/some-action", None, None).await;
        assert_eq!(status, StatusCode::OK);
    }

    #[tokio::test]
    async fn test_post_without_token_is_rejected() {
        let router = make_router();
        let status = call(router, axum::http::Method::POST, "/api/v1/some-action", None, None).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_post_with_valid_matching_token_passes() {
        let router = make_router();
        let status = call(
            router,
            axum::http::Method::POST,
            "/api/v1/some-action",
            Some("secret-token"),
            Some("secret-token"),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
    }

    #[tokio::test]
    async fn test_post_with_mismatched_token_is_rejected() {
        let router = make_router();
        let status = call(
            router,
            axum::http::Method::POST,
            "/api/v1/some-action",
            Some("cookie-token"),
            Some("header-token"),
        )
        .await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_post_with_empty_token_is_rejected() {
        let router = make_router();
        let status = call(
            router,
            axum::http::Method::POST,
            "/api/v1/some-action",
            Some(""),
            Some(""),
        )
        .await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_auth_nonce_route_skips_csrf() {
        let router = make_router();
        let status = call(
            router,
            axum::http::Method::POST,
            "/api/v1/auth/nonce",
            None,
            None,
        )
        .await;
        assert_eq!(status, StatusCode::OK);
    }
}
