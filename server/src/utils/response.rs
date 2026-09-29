//! # HTTP Response Utilities
//!
//! This module provides standardized response structures and helper functions
//! for creating consistent API responses across all endpoints.
//!
//! ## Response Format
//!
//! All successful responses follow this structure:
//! ```json
//! {
//!   "success": true,
//!   "data": { ... },
//!   "message": "Optional message"
//! }
//! ```
//!
//! Error responses use the flat `{ "code": "NOT_FOUND", "message": "..." }` shape
//! (see [`crate::utils::error::ApiError`]).

use crate::utils::error::ErrorCode;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;
use serde_json::Value;

/// Standard API response wrapper for successful responses
#[derive(Serialize)]
pub struct ApiResponse<T>
where
    T: Serialize,
{
    /// Always true for successful responses
    pub success: bool,
    /// Response data payload
    pub data: Option<T>,
    /// Optional success message
    pub message: Option<String>,
}

/// Error response body structure — flat `{ code, message }` shape.
#[derive(Serialize, utoipa::ToSchema)]
pub struct ApiErrorBody {
    /// Machine-readable error code.
    pub code: ErrorCode,
    /// Human-readable error message
    pub message: String,
}

/// Creates a successful response with data
///
/// # Arguments
/// * `data` - Serializable data to include in response
/// * `message` - Success message to include
///
/// # Returns
/// An Axum response with 200 status code and JSON body
pub fn success<T>(data: T, message: impl Into<String>) -> impl IntoResponse
where
    T: Serialize,
{
    let body = ApiResponse {
        success: true,
        data: Some(data),
        message: Some(message.into()),
    };
    (StatusCode::OK, Json(body))
}

/// Creates a successful response without data
///
/// # Arguments
/// * `message` - Success message to include
///
/// # Returns
/// An Axum response with 200 status code and JSON body
pub fn empty_success(message: impl Into<String>) -> impl IntoResponse {
    let body: ApiResponse<()> = ApiResponse {
        success: true,
        data: None,
        message: Some(message.into()),
    };
    (StatusCode::OK, Json(body))
}

/// Creates an error response with the standardised flat JSON body.
///
/// The `code` string argument is retained for call-site compatibility; the
/// JSON `code` is a machine-readable [`ErrorCode`] derived from `status`.
pub fn error(
    _code: &str,
    message: impl Into<String>,
    _details: Option<Value>,
    status: StatusCode,
) -> Response {
    let body = ApiErrorBody {
        code: ErrorCode::from_status(status),
        message: message.into(),
    };

    (status, Json(body)).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::header;
    use serde_json::json;

    #[tokio::test]
    async fn test_success_helper() {
        let payload = json!({ "id": 1, "name": "agora" });
        let resp = success(payload.clone(), "Operation succeeded").into_response();

        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(
            resp.headers()
                .get(header::CONTENT_TYPE)
                .and_then(|v| v.to_str().ok()),
            Some("application/json")
        );

        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();

        assert_eq!(body["success"], true);
        assert_eq!(body["data"], payload);
        assert_eq!(body["message"], "Operation succeeded");
    }

    #[tokio::test]
    async fn test_empty_success_helper() {
        let resp = empty_success("Deleted successfully").into_response();

        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(
            resp.headers()
                .get(header::CONTENT_TYPE)
                .and_then(|v| v.to_str().ok()),
            Some("application/json")
        );

        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();

        assert_eq!(body["success"], true);
        assert!(body["data"].is_null());
        assert_eq!(body["message"], "Deleted successfully");
    }

    #[tokio::test]
    async fn test_error_helper_bad_request() {
        let resp = error(
            "BAD_REQUEST",
            "Invalid payload provided",
            None,
            StatusCode::BAD_REQUEST,
        );

        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        assert_eq!(
            resp.headers()
                .get(header::CONTENT_TYPE)
                .and_then(|v| v.to_str().ok()),
            Some("application/json")
        );

        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();

        assert_eq!(body["code"], "VALIDATION_FAILED");
        assert_eq!(body["message"], "Invalid payload provided");
    }

    #[tokio::test]
    async fn test_error_helper_not_found() {
        let resp = error(
            "NOT_FOUND",
            "Resource not found",
            None,
            StatusCode::NOT_FOUND,
        );

        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
        assert_eq!(
            resp.headers()
                .get(header::CONTENT_TYPE)
                .and_then(|v| v.to_str().ok()),
            Some("application/json")
        );

        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();

        assert_eq!(body["code"], "NOT_FOUND");
        assert_eq!(body["message"], "Resource not found");
    }

    #[tokio::test]
    async fn test_error_helper_unauthorized() {
        let resp = error(
            "UNAUTHORIZED",
            "Missing authorization token",
            None,
            StatusCode::UNAUTHORIZED,
        );

        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();

        assert_eq!(body["code"], "UNAUTHORIZED");
        assert_eq!(body["message"], "Missing authorization token");
    }

    #[tokio::test]
    async fn test_error_helper_forbidden() {
        let resp = error(
            "FORBIDDEN",
            "Access denied",
            None,
            StatusCode::FORBIDDEN,
        );

        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();

        assert_eq!(body["code"], "FORBIDDEN");
        assert_eq!(body["message"], "Access denied");
    }

    #[tokio::test]
    async fn test_error_helper_internal_server_error() {
        let resp = error(
            "INTERNAL_SERVER_ERROR",
            "An unexpected error occurred",
            None,
            StatusCode::INTERNAL_SERVER_ERROR,
        );

        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();

        assert_eq!(body["code"], "INTERNAL_ERROR");
        assert_eq!(body["message"], "An unexpected error occurred");
    }

    #[tokio::test]
    async fn test_error_helper_conflict() {
        let resp = error(
            "CONFLICT",
            "Resource already exists",
            None,
            StatusCode::CONFLICT,
        );

        assert_eq!(resp.status(), StatusCode::CONFLICT);
        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();

        assert_eq!(body["code"], "CONFLICT");
        assert_eq!(body["message"], "Resource already exists");
    }
}
