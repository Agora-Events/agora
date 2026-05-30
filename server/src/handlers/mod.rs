pub mod auth;
pub mod categories;
pub mod events;
pub mod health;
pub mod leaderboard;
pub mod monitoring;
pub mod profile;
pub mod qr_payload;
pub mod rates;
pub mod soroban_listener;
pub mod upload;
pub mod ws;

use axum::{extract::Path, response::IntoResponse, response::Response};

use crate::utils::error::AppError;
use crate::utils::response::empty_success;

pub async fn example_validation_error() -> Response {
    AppError::ValidationError("The provided input is invalid".to_string()).into_response()
}

pub async fn example_not_found(Path(resource_id): Path<String>) -> Response {
    AppError::NotFound(format!("Resource with id '{}' was not found", resource_id)).into_response()
}

pub async fn example_empty_success() -> Response {
    empty_success("Operation completed successfully").into_response()
}
