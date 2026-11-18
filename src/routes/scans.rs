use crate::{db, models::IncomingScan, AppState};
use axum::{extract::State, http::StatusCode, Json};
use serde_json::{json, Value};

pub async fn ingest_scan(
    State(state): State<AppState>,
    Json(payload): Json<IncomingScan>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    if payload.set_code.trim().is_empty()
        || payload.collector_number.trim().is_empty()
        || payload.name.trim().is_empty()
    {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "set_code, collector_number, and name are required" })),
        ));
    }

    match db::record_scan(&state.pool, &payload).await {
        Ok(record) => {
            let foil_tag = if payload.is_foil { " [FOIL]" } else { "" };
            println!(
                "Scan recorded: [{} #{}]{} {} -> qty: {}",
                payload.set_code.to_uppercase(),
                payload.collector_number,
                foil_tag,
                payload.name,
                record.quantity
            );

            let bg_state = state.clone();
            let set_code = payload.set_code.clone();
            let collector_number = payload.collector_number.clone();
            let lang = payload.lang.clone();
            let is_foil = payload.is_foil;
            let card_id = record.id;

            tokio::spawn(async move {
                if let Ok(details) = bg_state
                    .scryfall
                    .fetch_card_details(&set_code, &collector_number, &lang, is_foil)
                    .await
                {
                    let _ = db::update_card_enrichment(
                        &bg_state.pool,
                        card_id,
                        details.price_usd,
                        details.price_clp,
                        details.image_url.as_deref(),
                    )
                    .await;
                }
            });

            Ok(Json(json!({
                "status": "recorded",
                "inventory_id": record.id,
                "quantity": record.quantity
            })))
        }
        Err(err) => {
            eprintln!("Database error recording scan: {err}");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to save scan to database" })),
            ))
        }
    }
}