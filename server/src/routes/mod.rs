use axum::{routing::get, Router};

use crate::config::{create_cors_layer, create_security_headers_layer};
use crate::handlers::health_check;

/// Creates the main application router with all routes and middleware.
///
/// # Middleware Stack (applied in order, bottom to top)
///
/// 1. Security Headers - Adds security headers to all responses
/// 2. CORS - Handles Cross-Origin Resource Sharing
///
/// # Routes
///
/// - `GET /health` - Health check endpoint
pub fn create_routes() -> Router {
    Router::new()
        .route("/health", get(health_check))
        // Apply security headers middleware
        .layer(create_security_headers_layer())
        // Apply CORS middleware (should be outermost to handle preflight)
        .layer(create_cors_layer())
}
