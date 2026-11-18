pub mod inventory;
pub mod scans;

use crate::AppState;
use axum::{
    routing::{get, patch, post},
    Router,
};

pub fn api_router() -> Router<AppState> {
    Router::new()
        .route("/api/scans", post(scans::ingest_scan))
        .route(
            "/api/cards",
            get(inventory::list_cards).delete(inventory::clear_inventory),
        )
        .route(
            "/api/cards/{id}",
            patch(inventory::adjust_stock).delete(inventory::remove_card),
        )
        .route("/api/prices/refresh", post(inventory::refresh_prices))
}