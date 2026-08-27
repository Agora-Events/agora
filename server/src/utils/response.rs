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
