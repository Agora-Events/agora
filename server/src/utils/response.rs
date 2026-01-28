use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;
use serde_json::Value;

#[derive(Serialize)]
pub struct ApiResponse<T>
where
    T: Serialize,
{
    pub success: bool,
    pub data: Option<T>,
    pub message: Option<String>,
}

#[derive(Serialize)]
pub struct ApiErrorBody {
    pub code: String,
    pub message: String,
    pub details: Option<Value>,
}

#[derive(Serialize)]
pub struct ApiErrorResponse {
    pub success: bool,
    pub error: ApiErrorBody,
}

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

pub fn empty_success(message: impl Into<String>) -> impl IntoResponse {
    let body: ApiResponse<()> = ApiResponse {
        success: true,
        data: None,
        message: Some(message.into()),
    };
    (StatusCode::OK, Json(body))
}

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
