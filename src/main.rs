mod db;
mod models;
mod routes;
mod scryfall;

use axum::{extract::State, routing::get, Json, Router};
use scryfall::ScryfallClient;
use serde_json::{json, Value};
use sqlx::SqlitePool;
use std::{env, net::SocketAddr};
use tower_http::services::ServeDir;

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub scryfall: ScryfallClient,
}

async fn health_check(State(state): State<AppState>) -> Json<Value> {
    let distinct_cards: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM inventory")
        .fetch_one(&state.pool)
        .await
        .unwrap_or(-1);

    Json(json!({
        "status": "ok",
        "service": "mtg-store-web",
        "database": "connected",
        "distinct_cards": distinct_cards
    }))
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let database_url =
        env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:store_inventory.db".to_string());
    let port: u16 = env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000);

    let pool = db::init_db(&database_url)
        .await
        .expect("Failed to initialize SQLite database");
    println!("Database ready: {}", database_url);

    let state = AppState {
        pool,
        scryfall: ScryfallClient::new(),
    };

    let app = Router::new()
        .route("/api/health", get(health_check))
        .merge(routes::api_router())
        .with_state(state)
        .fallback_service(ServeDir::new("static"));

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!("Server running at http://localhost:{}", port);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind port");

    axum::serve(listener, app)
        .await
        .expect("Server crashed");
}