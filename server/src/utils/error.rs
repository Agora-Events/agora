use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use thiserror::Error;
use tracing::error;

use crate::utils::response::error as error_response;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Authentication error: {0}")]
    AuthError(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Database error")]
    DatabaseError(#[from] sqlx::Error),

    #[error("External service error: {0}")]
    ExternalServiceError(String),

    #[error("Internal server error")]
    InternalServerError(String),
}

impl AppError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            AppError::ValidationError(_) => StatusCode::BAD_REQUEST,
            AppError::AuthError(_) => StatusCode::UNAUTHORIZED,
            AppError::Forbidden(_) => StatusCode::FORBIDDEN,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::ExternalServiceError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::InternalServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn code(&self) -> &'static str {
        match self {
            AppError::ValidationError(_) => "VALIDATION_ERROR",
            AppError::AuthError(_) => "AUTH_ERROR",
            AppError::Forbidden(_) => "FORBIDDEN",
            AppError::NotFound(_) => "NOT_FOUND",
            AppError::DatabaseError(_) => "DATABASE_ERROR",
            AppError::ExternalServiceError(_) => "EXTERNAL_SERVICE_ERROR",
            AppError::InternalServerError(_) => "INTERNAL_SERVER_ERROR",
        }
    }

    fn log(&self) {
        match self {
            AppError::ValidationError(msg)
            | AppError::AuthError(msg)
            | AppError::Forbidden(msg)
            | AppError::NotFound(msg)
            | AppError::ExternalServiceError(msg)
            | AppError::InternalServerError(msg) => {
                error!(error = ?self, message = %msg, "Application error");
            }
            AppError::DatabaseError(e) => {
                error!(error = ?e, "Database error");
            }
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let code = self.code();

        // Log internal details
        self.log();

        // Only expose high-level message to the client
        let public_message = match &self {
            AppError::ValidationError(msg)
            | AppError::AuthError(msg)
            | AppError::Forbidden(msg)
            | AppError::NotFound(msg)
            | AppError::ExternalServiceError(msg)
            | AppError::InternalServerError(msg) => msg.clone(),
            AppError::DatabaseError(_) => "A database error occurred".to_string(),
        };

        // Do not expose internal details in the API response
        let details = None;

        error_response(code, public_message, details, status)
    }
}
