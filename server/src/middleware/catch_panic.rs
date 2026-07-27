//! Panic-catching middleware that converts uncaught panics into standardised
//! [`ApiError`] JSON responses (`{ "code": 500, "message": "..." }`).

use axum::response::{IntoResponse, Response};
use tower_http::catch_panic::CatchPanicLayer;

use crate::utils::error::ApiError;

/// Build a [`CatchPanicLayer`] that maps panics to a 500 [`ApiError`] JSON body.
pub fn catch_panic_layer() -> CatchPanicLayer<fn(Box<dyn std::any::Any + Send>) -> Response> {
    CatchPanicLayer::custom(handle_panic as fn(Box<dyn std::any::Any + Send>) -> Response)
}

fn handle_panic(err: Box<dyn std::any::Any + Send>) -> Response {
    let detail = if let Some(s) = err.downcast_ref::<&str>() {
        (*s).to_string()
    } else if let Some(s) = err.downcast_ref::<String>() {
        s.clone()
    } else {
        "unknown panic".to_string()
    };
    tracing::error!(panic = %detail, "Uncaught panic in request handler");
    ApiError::internal("Internal server error").into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::{to_bytes, Body};
    use axum::http::{Request, StatusCode};
    use axum::routing::get;
    use axum::Router;
    use tower::ServiceExt;

    async fn panicking_handler() -> &'static str {
        panic!("boom");
    }

    #[tokio::test]
    async fn test_catch_panic_returns_api_error_json() {
        let app = Router::new()
            .route("/panic", get(panicking_handler))
            .layer(catch_panic_layer());

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/panic")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["code"], 500);
        assert_eq!(json["message"], "Internal server error");
    }
}
