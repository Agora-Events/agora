pub mod auth;
pub mod categories;
pub mod events;
pub mod health;
pub mod leaderboard;
pub mod marketplace;
pub mod monitoring;
pub mod profile;
pub mod qr_payload;
pub mod rates;
pub mod soroban_listener;
pub mod waiting_room;
pub mod ws;

use axum::{extract::Path, response::IntoResponse, response::Response};

use crate::utils::error::{ApiError, AppError};
use crate::utils::response::empty_success;

/// Example endpoint demonstrating the standardised [`ApiError`] response shape.
pub async fn example_validation_error() -> Result<(), ApiError> {
    Err(ApiError::from(AppError::ValidationError(
        "The provided input is invalid".to_string(),
    )))
}

/// Example endpoint demonstrating a not-found [`ApiError`].
pub async fn example_not_found(Path(resource_id): Path<String>) -> Result<(), ApiError> {
    Err(ApiError::from(AppError::NotFound(format!(
        "Resource with id '{}' was not found",
        resource_id
    ))))
}

pub async fn example_empty_success() -> Response {
    empty_success("Operation completed successfully").into_response()
}
