use axum::Router;
use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use tokio::net::TcpListener;

use agora_server::config::request_id::REQUEST_ID_HEADER;
use agora_server::config::Config;
use agora_server::routes::create_routes;
use agora_server::utils::logging::init_logging;

#[tokio::main]
async fn main() {
    dotenv().ok();
    init_logging();

    let config = Config::from_env().expect("Failed to load configuration");
    tracing::info!("Starting server in {} mode", config.rust_env);
    tracing::info!("Configuration: PORT={}", config.port);
    tracing::info!("Configuration: RUST_ENV={}", config.rust_env);
    tracing::info!("Configuration: RUST_LOG={}", config.rust_log);
    tracing::info!(
        "Configuration: CORS_ALLOWED_ORIGINS={}",
        config.cors_allowed_origins
    );
    // Note: DATABASE_URL is strictly excluded from logging for security reasons.

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await
        .expect("Failed to connect to database");

    tracing::info!("Successfully connected to database");

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    tracing::info!("Migrations run successfully");

    let app: Router = create_routes(pool.clone());
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    tracing::info!("🚀 Server running at http://localhost:{}", config.port);
    tracing::info!("Request IDs will be set via '{REQUEST_ID_HEADER}' header");

    let listener = TcpListener::bind(addr)
        .await
        .expect("Failed to bind address");

    axum::serve(listener, app).await.expect("Server failed");
}
