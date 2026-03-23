use axum::Router;
use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::env;
use std::net::SocketAddr;
use tokio::net::TcpListener;

use agora_server::routes::create_routes;
use agora_server::utils::logging::init_logging;

#[tokio::main]
async fn main() {
    dotenv().ok();
    init_logging();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    tracing::info!("Successfully connected to database");

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    tracing::info!("Migrations run successfully");

    let app: Router = create_routes(pool.clone());

    let addr = SocketAddr::from(([0, 0, 0, 0], 3001));
    tracing::info!("🚀 Server running at http://{}", addr);

    let listener = TcpListener::bind(addr)
        .await
        .expect("Failed to bind address");

    axum::serve(listener, app).await.expect("Server failed");
}
