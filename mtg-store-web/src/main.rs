use axum::{routing::get, Json, Router};
use serde_json::{json, Value};
use std::net::SocketAddr;
use tower_http::services::ServeDir;

async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "service": "mtg-store-web"
    }))
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/api/health", get(health_check))
        .fallback_service(ServeDir::new("static"));

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("Server running at http://localhost:3000");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind to port 3000");

    axum::serve(listener, app)
        .await
        .expect("Server crashed");
}