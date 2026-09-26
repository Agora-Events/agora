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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{header, Method, Request, StatusCode},
        middleware,
        routing::get,
        Router,
    };
    use tower::ServiceExt;

    fn build_test_router() -> Router {
        async fn dummy_handler() -> &'static str {
            "ok"
        }

        Router::new()
            .route(
                "/api/test",
                get(dummy_handler)
                    .post(dummy_handler)
                    .put(dummy_handler)
                    .delete(dummy_handler)
                    .options(dummy_handler),
            )
            .route("/api/auth/nonce", get(dummy_handler).post(dummy_handler))
            .route("/api/auth/verify", get(dummy_handler).post(dummy_handler))
            .layer(middleware::from_fn(check_csrf))
    }

    #[tokio::test]
    async fn test_safe_methods_pass_without_token() {
        let app = build_test_router();

        // GET passes
        let req_get = Request::builder()
            .method(Method::GET)
            .uri("/api/test")
            .body(Body::empty())
            .unwrap();
        let resp = app.clone().oneshot(req_get).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        // HEAD passes
        let req_head = Request::builder()
            .method(Method::HEAD)
            .uri("/api/test")
            .body(Body::empty())
            .unwrap();
        let resp = app.clone().oneshot(req_head).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        // OPTIONS passes
        let req_options = Request::builder()
            .method(Method::OPTIONS)
            .uri("/api/test")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req_options).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_unsafe_methods_without_token_are_rejected() {
        let app = build_test_router();

        // POST without token is rejected
        let req_post = Request::builder()
            .method(Method::POST)
            .uri("/api/test")
            .body(Body::empty())
            .unwrap();
        let resp = app.clone().oneshot(req_post).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

        // PUT without token is rejected
        let req_put = Request::builder()
            .method(Method::PUT)
            .uri("/api/test")
            .body(Body::empty())
            .unwrap();
        let resp = app.clone().oneshot(req_put).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

        // DELETE without token is rejected
        let req_delete = Request::builder()
            .method(Method::DELETE)
            .uri("/api/test")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req_delete).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_post_with_valid_matching_token_passes() {
        let app = build_test_router();

        let req = Request::builder()
            .method(Method::POST)
            .uri("/api/test")
            .header(header::COOKIE, "XSRF-TOKEN=secret_token_123")
            .header("X-XSRF-TOKEN", "secret_token_123")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_post_with_mismatched_token_is_rejected() {
        let app = build_test_router();

        let req = Request::builder()
            .method(Method::POST)
            .uri("/api/test")
            .header(header::COOKIE, "XSRF-TOKEN=cookie_token_abc")
            .header("X-XSRF-TOKEN", "header_token_xyz")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_post_with_missing_cookie_or_header_token_is_rejected() {
        let app = build_test_router();

        // Cookie present, header missing
        let req = Request::builder()
            .method(Method::POST)
            .uri("/api/test")
            .header(header::COOKIE, "XSRF-TOKEN=token_123")
            .body(Body::empty())
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

        // Header present, cookie missing
        let req = Request::builder()
            .method(Method::POST)
            .uri("/api/test")
            .header("X-XSRF-TOKEN", "token_123")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_auth_endpoints_bypass_csrf() {
        let app = build_test_router();

        // /api/auth/nonce without token passes
        let req_nonce = Request::builder()
            .method(Method::POST)
            .uri("/api/auth/nonce")
            .body(Body::empty())
            .unwrap();
        let resp = app.clone().oneshot(req_nonce).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        // /api/auth/verify without token passes
        let req_verify = Request::builder()
            .method(Method::POST)
            .uri("/api/auth/verify")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req_verify).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }
}
