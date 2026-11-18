use crate::{db, models::CardStockItem, AppState};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Debug, Deserialize)]
pub struct AdjustStockRequest {
    pub delta: i64,
}

pub async fn list_cards(
    State(state): State<AppState>,
) -> Result<Json<Vec<CardStockItem>>, (StatusCode, Json<Value>)> {
    db::get_all_cards(&state.pool).await.map(Json).map_err(|err| {
        eprintln!("Database error fetching cards: {err}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Failed to fetch inventory" })),
        )
    })
}

pub async fn adjust_stock(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<AdjustStockRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match db::adjust_quantity(&state.pool, id, body.delta).await {
        Ok(Some(new_qty)) => Ok(Json(json!({
            "id": id,
            "quantity": new_qty
        }))),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Card not found" })),
        )),
        Err(err) => {
            eprintln!("Database error updating quantity: {err}");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to update quantity" })),
            ))
        }
    }
}

pub async fn remove_card(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match db::delete_card(&state.pool, id).await {
        Ok(true) => Ok(Json(json!({ "deleted": id }))),
        Ok(false) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Card not found" })),
        )),
        Err(err) => {
            eprintln!("Database error deleting card: {err}");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to delete card" })),
            ))
        }
    }
}

pub async fn refresh_prices(State(state): State<AppState>) -> Json<Value> {
    let updated = state.scryfall.refresh_all_prices(&state.pool).await;
    Json(json!({
        "status": "completed",
        "updated_cards": updated
    }))
}



pub async fn clear_inventory(
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match db::clear_inventory(&state.pool).await {
        Ok(()) => {
            println!("Inventario reiniciado desde cero.");
            Ok(Json(json!({ "status": "cleared" })))
        }
        Err(err) => {
            eprintln!("Database error clearing inventory: {err}");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to clear inventory" })),
            ))
        }
    }
}