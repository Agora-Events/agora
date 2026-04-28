//! # Agora Server Main Entry Point
//!
//! This module contains the main entry point for the Agora events platform server.
//! It initializes and configures all necessary services including:
//! - Database connectivity and migrations
//! - HTTP server with routing
//! - Logging and configuration management
//! - CORS and security middleware
//!
//! The server is built using Axum framework and connects to a PostgreSQL database.

use axum::Router;
use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use tokio::net::TcpListener;

use agora_server::config::request_id::REQUEST_ID_HEADER;
use agora_server::config::Config;
use agora_server::utils::logging::init_logging;

/// Main application entry point.
///
/// Initializes the server by:
/// 1. Loading environment variables from .env file
/// 2. Setting up structured logging
/// 3. Loading configuration from environment
/// 4. Establishing database connection pool
/// 5. Running database migrations
/// 6. Starting the HTTP server with configured routes
#[tokio::main]
async fn main() {
    dotenv().ok();
    init_logging();

    let config = Config::from_env().expect("Failed to load configuration");
    tracing::info!("Starting server in {} mode", config.rust_env);
    tracing::info!("Configuration: PORT={}", config.port);
    tracing::info!("Configuration: RUST_ENV={}", config.rust_env);
    tracing::info!("Configuration: RUST_LOG={}", config.rust_log);
    tracing::info!("Configuration: CORS_ALLOWED_ORIGINS={}", config.cors_allowed_origins);
    tracing::info!("Configuration: SOROBAN_RPC_URL={}", config.soroban_rpc_url);
    tracing::info!("Configuration: REDIS_URL={}", config.redis_url);
    // Note: DATABASE_URL is strictly excluded from logging for security reasons.

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url).await
        .expect("Failed to connect to database");

    tracing::info!("Successfully connected to database");

    sqlx::migrate!().run(&pool).await.expect("Failed to run migrations");

    tracing::info!("Migrations run successfully");

    // Initialize Redis cache
    let redis = match agora_server::cache::RedisCache::new(&config.redis_url).await {
        Ok(redis) => {
            tracing::info!("Successfully connected to Redis at {}", config.redis_url);
            redis
        }
        Err(e) => {
            tracing::error!("Failed to connect to Redis: {:?}", e);
            tracing::warn!("Continuing without Redis cache - performance may be degraded");
            panic!("Redis connection required for caching");
        }
    };

    let app: Router = agora_server::routes::create_routes(pool.clone(), config.clone(), redis).await;
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    tracing::info!("🚀 Server running at http://localhost:{}", config.port);
    tracing::info!("Request IDs will be set via '{REQUEST_ID_HEADER}' header");

    let listener = TcpListener::bind(addr).await.expect("Failed to bind address");

    axum::serve(listener, app).await.expect("Server failed");
}
