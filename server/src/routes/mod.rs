use axum::{routing::get, Router};

use crate::handlers::health_check;

pub fn create_routes() -> Router {
    Router::new().route("/health", get(health_check))
}
