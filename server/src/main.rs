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
//!
//! On `SIGTERM`/`SIGINT` the server performs a graceful shutdown (Issue #1261):
//! in-flight requests are allowed to finish (up to `SHUTDOWN_TIMEOUT_SECS`),
//! background tasks are signalled to stop via a cancellation token, and the
//! database pool and Redis connection are closed explicitly before exit.

use std::net::SocketAddr;
use std::time::{Duration, Instant};

use axum::Router;
use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use tokio::net::TcpListener;
use tokio::signal::unix::{signal, SignalKind};
use tokio_util::sync::CancellationToken;

use agora_server::cache::RedisCache;
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

    let config = Config::from_env().unwrap_or_else(|e| {
        eprintln!("ERROR: Failed to load configuration: {e}");
        std::process::exit(1);
    });

    if let Err(e) = config.validate() {
        eprintln!("ERROR: Invalid configuration:\n{e}");
        tracing::error!("Server startup aborted due to configuration errors:\n{e}");
        std::process::exit(1);
    }

    tracing::info!("Starting server in {} mode", config.rust_env);
    tracing::info!("Configuration: PORT={}", config.port);
    tracing::info!("Configuration: RUST_ENV={}", config.rust_env);
    tracing::info!("Configuration: RUST_LOG={}", config.rust_log);
    tracing::info!(
        "Configuration: CORS_ALLOWED_ORIGINS={}",
        config.cors_allowed_origins
    );
    tracing::info!("Configuration: SOROBAN_RPC_URL={}", config.soroban_rpc_url);
    tracing::info!("Configuration: REDIS_URL={}", config.redis_url);
    // Note: DATABASE_URL is strictly excluded from logging for security reasons.

    let pool = PgPoolOptions::new()
        .max_connections(config.db_max_connections)
        .min_connections(config.db_min_connections)
        .acquire_timeout(std::time::Duration::from_secs(config.db_acquire_timeout_secs))
        .idle_timeout(std::time::Duration::from_secs(config.db_idle_timeout_secs))
        .connect(&config.database_url)
        .await
        .expect("Failed to connect to database");

    tracing::info!(
        "Database pool: max_connections={} min_connections={} acquire_timeout={}s idle_timeout={}s",
        config.db_max_connections,
        config.db_min_connections,
        config.db_acquire_timeout_secs,
        config.db_idle_timeout_secs,
    );
    tracing::info!("Successfully connected to database");

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    tracing::info!("Migrations run successfully");

    // Validate categories match contract (Issue #1076)
    let categories_synced =
        agora_server::handlers::categories::validate_categories_match_contract(&pool).await;
    agora_server::handlers::health::set_category_sync_status(categories_synced);
    if !categories_synced {
        tracing::error!("Category sync validation failed - database categories do not match contract canonical list");
    }

    // Initialize Redis cache
    let redis = match RedisCache::new(&config.redis_url).await {
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

    // Shared cancellation token used to signal background tasks to stop during a
    // graceful shutdown (Issue #1261).
    let shutdown = CancellationToken::new();

    let app: Router =
        agora_server::routes::create_routes(pool.clone(), config.clone(), redis.clone(), shutdown.clone()).await;
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    tracing::info!("🚀 Server running at http://localhost:{}", config.port);
    tracing::info!("Request IDs will be set via '{REQUEST_ID_HEADER}' header");

    // Spawn periodic background task to clean up old nonces (Issue #823)
    let cleanup_pool = pool.clone();
    let cleanup_shutdown = shutdown.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(15 * 60)); // 15 minutes
        loop {
            tokio::select! {
                _ = cleanup_shutdown.cancelled() => {
                    tracing::info!("Nonce cleanup task stopping");
                    break;
                }
                _ = interval.tick() => {
                    tracing::info!("Running periodic cleanup of jwt_nonces...");
                    match sqlx::query(
                        "DELETE FROM jwt_nonces WHERE expires_at < NOW() OR (used = TRUE AND created_at < NOW() - INTERVAL '7 days')"
                    )
                    .execute(&cleanup_pool)
                    .await
                    {
                        Ok(result) => {
                            if result.rows_affected() > 0 {
                                tracing::info!("Cleaned up {} expired/used nonces.", result.rows_affected());
                            }
                        }
                        Err(e) => {
                            tracing::error!("Failed to clean up jwt_nonces: {:?}", e);
                        }
                    }
                }
            }
        }
    });

    let listener = TcpListener::bind(addr)
        .await
        .expect("Failed to bind address");

    let shutdown_signal = shutdown.clone();
    let shutdown_timeout = config.shutdown_timeout_secs;
    let start = Instant::now();

    let serve_result = axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            // Resolve on SIGTERM or SIGINT. The OS sends SIGTERM on `docker stop`
            // / Kubernetes rolling deploys, and SIGINT on Ctrl-C.
            let mut sigterm =
                signal(SignalKind::terminate()).expect("failed to install SIGTERM handler");
            let mut sigint =
                signal(SignalKind::interrupt()).expect("failed to install SIGINT handler");

            tokio::select! {
                _ = sigterm.recv() => {
                    tracing::info!("Received SIGTERM, starting graceful shutdown");
                }
                _ = sigint.recv() => {
                    tracing::info!("Received SIGINT, starting graceful shutdown");
                }
                _ = tokio::time::sleep(Duration::from_secs(shutdown_timeout)) => {
                    tracing::warn!(
                        "Shutdown timeout of {shutdown_timeout}s elapsed, forcing shutdown"
                    );
                }
            }

            // Signal background tasks (indexer, queue admission worker, nonce
            // cleanup) to stop via the cancellation token.
            shutdown_signal.cancel();
        })
        .await;

    if let Err(e) = serve_result {
        tracing::error!("Server error during shutdown: {e}");
    }

    // Close connections explicitly (Issue #1261). In-flight requests have already
    // drained by the time `axum::serve` resolved.
    pool.close().await;
    redis.close().await;

    tracing::info!("shutdown complete in {:?}", start.elapsed());
    std::process::exit(0);
}
