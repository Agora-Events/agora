use axum::{routing::get, Router};

use crate::config::{create_cors_layer, create_security_headers_layer};
use crate::handlers::health_check;

pub fn create_routes() -> Router {
    Router::new()
        .route("/health", get(health_check))
        .layer(create_security_headers_layer())
        .layer(create_cors_layer())
}
