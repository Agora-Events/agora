use axum::{extract::Path, response::IntoResponse, response::Response};
use serde::Serialize;

use crate::utils::error::AppError;
use crate::utils::response::{empty_success, success};

#[derive(Serialize)]
struct HealthPayload {
    status: &'static str,
    service: &'static str,
}

pub async fn health_check() -> Response {
    let payload = HealthPayload {
        status: "ok",
        service: "agora-api",
    };

    success(payload, "Health check successful").into_response()
}

pub async fn example_validation_error() -> Response {
    AppError::ValidationError("The provided input is invalid".to_string()).into_response()
}

pub async fn example_not_found(Path(resource_id): Path<String>) -> Response {
    AppError::NotFound(format!("Resource with id '{}' was not found", resource_id)).into_response()
}

pub async fn example_empty_success() -> Response {
    empty_success("Operation completed successfully").into_response()
}
