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
