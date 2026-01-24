use axum::Router;
use dotenvy::dotenv;
use std::net::SocketAddr;
use tokio::net::TcpListener;

use agora_server::routes::create_routes;

#[tokio::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    let app: Router = create_routes();

    let addr = SocketAddr::from(([0, 0, 0, 0], 3001));
    tracing::info!("🚀 Server running at http://{}", addr);

    let listener = TcpListener::bind(addr)
        .await
        .expect("Failed to bind address");

    axum::serve(listener, app).await.expect("Server failed");
}
