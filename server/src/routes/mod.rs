use axum::{routing::get, Router};

use crate::config::{create_cors_layer, create_security_headers_layer};
use crate::handlers::{
    example_empty_success, example_not_found, example_validation_error, health_check,
};

pub fn create_routes() -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/examples/validation-error", get(example_validation_error))
        .route("/examples/empty-success", get(example_empty_success))
        .route("/examples/not-found/:id", get(example_not_found))
        .layer(create_security_headers_layer())
        .layer(create_cors_layer())
}
