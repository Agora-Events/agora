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
//! Error responses are handled by the `AppError` type but can also
//! be created manually using the `error` function.

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

/// Error response body structure
#[derive(Serialize)]
pub struct ApiErrorBody {
    /// Machine-readable error code
    pub code: String,
    /// Human-readable error message
    pub message: String,
    /// Optional error details
    pub details: Option<Value>,
}

/// Complete error response wrapper
#[derive(Serialize)]
pub struct ApiErrorResponse {
    /// Always false for error responses
    pub success: bool,
    /// Error details
    pub error: ApiErrorBody,
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

/// Creates an error response with custom status code
///
/// # Arguments
/// * `code` - Error code string
/// * `message` - Error message
/// * `details` - Optional error details
/// * `status` - HTTP status code
///
/// # Returns
/// An Axum response with specified status code and JSON error body
pub fn error(
    code: &str,
    message: impl Into<String>,
    details: Option<Value>,
    status: StatusCode,
) -> Response {
    let body = ApiErrorResponse {
        success: false,
        error: ApiErrorBody {
            code: code.to_string(),
            message: message.into(),
            details,
        },
    };

    (status, Json(body)).into_response()
}
