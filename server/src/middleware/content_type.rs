use axum::{
    body::Body,
    http::{header::CONTENT_LENGTH, header::CONTENT_TYPE, Method, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde_json::json;

/// Middleware that enforces `Content-Type: application/json` on POST and PUT
/// requests that carry a body. Requests with no body (Content-Length: 0 or
/// absent) are allowed through so that body-less endpoints (e.g. POST /logout)
/// are not affected.
///
/// Returns HTTP 415 Unsupported Media Type when the header is missing or does
/// not start with `application/json`.
pub async fn require_json_content_type(req: Request<Body>, next: Next) -> Response {
    if matches!(req.method(), &Method::POST | &Method::PUT) {
        let content_length = req
            .headers()
            .get(CONTENT_LENGTH)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u64>().ok());

        let has_body = match content_length {
            Some(len) => len > 0,
            None => req
                .headers()
                .contains_key(axum::http::header::TRANSFER_ENCODING),
        };

        if has_body {
            let has_json_ct = req
                .headers()
                .get(CONTENT_TYPE)
                .and_then(|v| v.to_str().ok())
                .map(|v| v.starts_with("application/json"))
                .unwrap_or(false);

            if !has_json_ct {
                let body = json!({
                    "success": false,
                    "error": {
                        "code": "UNSUPPORTED_MEDIA_TYPE",
                        "message": "Content-Type must be application/json"
                    }
                });
                return (StatusCode::UNSUPPORTED_MEDIA_TYPE, axum::Json(body)).into_response();
            }
        }
    }

    next.run(req).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        middleware,
        routing::post,
        Router,
    };
    use tower::ServiceExt;

    fn test_router() -> Router {
        Router::new()
            .route("/test", post(|| async { "ok" }))
            .layer(middleware::from_fn(require_json_content_type))
    }

    async fn send(router: Router, method: &str, content_type: Option<&str>, body: &str) -> StatusCode {
        let mut builder = Request::builder().method(method).uri("/test");
        if let Some(ct) = content_type {
            builder = builder.header("content-type", ct);
        }
        if !body.is_empty() {
            builder = builder.header("content-length", body.len().to_string());
        }
        let req = builder.body(Body::from(body.to_string())).unwrap();
        router.oneshot(req).await.unwrap().status()
    }

    #[tokio::test]
    async fn test_post_with_json_content_type_passes() {
        let router = test_router();
        let status = send(router, "POST", Some("application/json"), r#"{"x":1}"#).await;
        assert_ne!(status, StatusCode::UNSUPPORTED_MEDIA_TYPE);
    }

    #[tokio::test]
    async fn test_post_with_wrong_content_type_returns_415() {
        let router = test_router();
        let status = send(router, "POST", Some("text/plain"), r#"hello"#).await;
        assert_eq!(status, StatusCode::UNSUPPORTED_MEDIA_TYPE);
    }

    #[tokio::test]
    async fn test_post_missing_content_type_with_body_returns_415() {
        let router = test_router();
        let status = send(router, "POST", None, r#"{"x":1}"#).await;
        assert_eq!(status, StatusCode::UNSUPPORTED_MEDIA_TYPE);
    }

    #[tokio::test]
    async fn test_post_no_body_passes_without_content_type() {
        let router = test_router();
        let status = send(router, "POST", None, "").await;
        assert_ne!(status, StatusCode::UNSUPPORTED_MEDIA_TYPE);
    }

    #[tokio::test]
    async fn test_json_with_charset_passes() {
        let router = test_router();
        let status = send(router, "POST", Some("application/json; charset=utf-8"), r#"{"x":1}"#).await;
        assert_ne!(status, StatusCode::UNSUPPORTED_MEDIA_TYPE);
    }

    #[tokio::test]
    async fn test_get_request_not_affected() {
        let router = Router::new()
            .route("/test", axum::routing::get(|| async { "ok" }))
            .layer(middleware::from_fn(require_json_content_type));

        let req = Request::builder()
            .method("GET")
            .uri("/test")
            .body(Body::empty())
            .unwrap();
        let status = router.oneshot(req).await.unwrap().status();
        assert_ne!(status, StatusCode::UNSUPPORTED_MEDIA_TYPE);
    }
}
