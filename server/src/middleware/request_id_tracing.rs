//! # Request ID Tracing Middleware
//!
//! Extracts the `x-request-id` header (set by [`SetRequestIdLayer`]) and
//! inserts it as a field on the current tracing span so that every log line
//! emitted while handling a request automatically carries the request ID.
//!
//! This satisfies issue #498: a single request ID can be used to find all
//! logs related to a specific user action.

use axum::{extract::Request, middleware::Next, response::Response};
use tracing::Instrument;

use crate::config::request_id::REQUEST_ID_HEADER;

/// Axum middleware that injects `request_id` into the tracing span.
///
/// Must be applied **after** [`SetRequestIdLayer`] so the header is already
/// present on the request when this middleware runs.
pub async fn trace_request_id(request: Request, next: Next) -> Response {
    let request_id = request
        .headers()
        .get(REQUEST_ID_HEADER)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_owned();

    let span = tracing::info_span!("request", request_id = %request_id);
    next.run(request).instrument(span).await
}

/// Axum middleware that copies `x-request-id` from the request headers to the
/// response. This guarantees the header appears on every response — including
/// error responses — even when the inner handler creates a fresh `Response`
/// that does not carry the header.
///
/// Must be applied after [`SetRequestIdLayer`] so the header is already set on
/// the incoming request.
pub async fn propagate_request_id(request: Request, next: Next) -> Response {
    let request_id = request
        .headers()
        .get(REQUEST_ID_HEADER)
        .cloned();

    let mut response = next.run(request).await;

    if let Some(id) = request_id {
        let header_name = axum::http::HeaderName::from_static(REQUEST_ID_HEADER);
        response.headers_mut().entry(header_name).or_insert(id);
    }

    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request, http::StatusCode, middleware, routing::get, Router};
    use crate::config::request_id::set_request_id_layer;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_propagate_request_id_on_success_response() {
        let router = Router::new()
            .route("/", get(|| async { "ok" }))
            .layer(middleware::from_fn(propagate_request_id))
            .layer(set_request_id_layer());

        let custom_id = "test-id-success";
        let req = Request::builder()
            .uri("/")
            .header(REQUEST_ID_HEADER, custom_id)
            .body(Body::empty())
            .unwrap();
        let resp = router.oneshot(req).await.unwrap();

        assert_eq!(
            resp.headers()
                .get(REQUEST_ID_HEADER)
                .and_then(|v| v.to_str().ok()),
            Some(custom_id)
        );
    }

    #[tokio::test]
    async fn test_propagate_request_id_on_error_response() {
        let router = Router::new()
            .route(
                "/",
                get(|| async { StatusCode::BAD_REQUEST }),
            )
            .layer(middleware::from_fn(propagate_request_id))
            .layer(set_request_id_layer());

        let custom_id = "test-id-error";
        let req = Request::builder()
            .uri("/")
            .header(REQUEST_ID_HEADER, custom_id)
            .body(Body::empty())
            .unwrap();
        let resp = router.oneshot(req).await.unwrap();

        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        assert_eq!(
            resp.headers()
                .get(REQUEST_ID_HEADER)
                .and_then(|v| v.to_str().ok()),
            Some(custom_id)
        );
    }
}
