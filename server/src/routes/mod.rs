use axum::{routing::{get, post}, Router};
use sqlx::PgPool;

use crate::config::{create_cors_layer, create_security_headers_layer};
use crate::handlers::{
    example_empty_success, example_not_found, example_validation_error,
    health::{health_check, health_check_db, health_check_ready},
    refund::{handle_event_cancellation, request_guest_refund},
};

pub fn create_routes(pool: PgPool) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/health/db", get(health_check_db))
        .route("/health/ready", get(health_check_ready))
        .route("/examples/validation-error", get(example_validation_error))
        .route("/examples/empty-success", get(example_empty_success))
        .route("/examples/not-found/:id", get(example_not_found))
        .route("/refunds/guest/tickets/:ticket_id", post(request_guest_refund))
        .route("/events/:event_id/cancel", post(handle_event_cancellation))
        .with_state(pool)
        .layer(create_security_headers_layer())
        .layer(create_cors_layer())
}
