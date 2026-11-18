use crate::models::{CardStockItem, IncomingScan};
use sqlx::{sqlite::SqliteConnectOptions, FromRow, SqlitePool};
use std::str::FromStr;

#[derive(FromRow)]
pub struct ScanRecordResult {
    pub id: i64,
    pub quantity: i64,
}

pub async fn init_db(database_url: &str) -> Result<SqlitePool, sqlx::Error> {
    let options = SqliteConnectOptions::from_str(database_url)?
        .create_if_missing(true);

    let pool = SqlitePool::connect_with(options).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}

pub async fn record_scan(
    pool: &SqlitePool,
    scan: &IncomingScan,
) -> Result<ScanRecordResult, sqlx::Error> {
    let set_code = scan.set_code.trim().to_lowercase();
    let collector_number = scan.collector_number.trim().to_string();
    let lang = scan.lang.trim().to_lowercase();
    let colors_json = serde_json::to_string(&scan.colors).unwrap_or_else(|_| "[]".to_string());

    let result = sqlx::query_as::<_, ScanRecordResult>(
        r#"
        INSERT INTO inventory (
            set_code, collector_number, lang, is_foil, name, name_en, name_es,
            rarity, colors, image_url, bin_number, quantity, last_updated
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, 1, datetime('now'))
        ON CONFLICT(set_code, collector_number, lang, is_foil) DO UPDATE SET
            quantity = inventory.quantity + 1,
            bin_number = COALESCE(excluded.bin_number, inventory.bin_number),
            image_url = COALESCE(excluded.image_url, inventory.image_url),
            name_en = COALESCE(excluded.name_en, inventory.name_en),
            name_es = COALESCE(excluded.name_es, inventory.name_es),
            last_updated = datetime('now')
        RETURNING id, quantity
        "#,
    )
    .bind(&set_code)
    .bind(&collector_number)
    .bind(&lang)
    .bind(scan.is_foil)
    .bind(&scan.name)
    .bind(&scan.name_en)
    .bind(&scan.name_es)
    .bind(&scan.rarity)
    .bind(&colors_json)
    .bind(&scan.image_url)
    .bind(scan.bin_number)
    .fetch_one(pool)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO scan_log (inventory_id, source, bin_number)
        VALUES (?1, ?2, ?3)
        "#,
    )
    .bind(result.id)
    .bind(&scan.source)
    .bind(scan.bin_number)
    .execute(pool)
    .await?;

    Ok(result)
}

pub async fn get_all_cards(pool: &SqlitePool) -> Result<Vec<CardStockItem>, sqlx::Error> {
    sqlx::query_as::<_, CardStockItem>(
        r#"
        SELECT
            id, set_code, collector_number, lang, is_foil, name, name_en, name_es,
            rarity, colors, price_usd, price_clp, image_url, bin_number, quantity, last_updated
        FROM inventory
        ORDER BY last_updated DESC, id DESC
        "#,
    )
    .fetch_all(pool)
    .await
}

pub async fn adjust_quantity(
    pool: &SqlitePool,
    id: i64,
    delta: i64,
) -> Result<Option<i64>, sqlx::Error> {
    let new_qty: Option<i64> = sqlx::query_scalar(
        r#"
        UPDATE inventory
        SET quantity = quantity + ?1,
            last_updated = datetime('now')
        WHERE id = ?2
        RETURNING quantity
        "#,
    )
    .bind(delta)
    .bind(id)
    .fetch_optional(pool)
    .await?;

    match new_qty {
        Some(qty) if qty <= 0 => {
            sqlx::query("DELETE FROM inventory WHERE id = ?1")
                .bind(id)
                .execute(pool)
                .await?;
            Ok(Some(0))
        }
        other => Ok(other),
    }
}

pub async fn delete_card(pool: &SqlitePool, id: i64) -> Result<bool, sqlx::Error> {
    let res = sqlx::query("DELETE FROM inventory WHERE id = ?1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(res.rows_affected() > 0)
}

pub async fn update_card_enrichment(
    pool: &SqlitePool,
    id: i64,
    price_usd: Option<f64>,
    price_clp: Option<i64>,
    image_url: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE inventory
        SET price_usd = COALESCE(?1, price_usd),
            price_clp = COALESCE(?2, price_clp),
            image_url = COALESCE(?3, image_url)
        WHERE id = ?4
        "#,
    )
    .bind(price_usd)
    .bind(price_clp)
    .bind(image_url)
    .bind(id)
    .execute(pool)
    .await?;

    Ok(())
}


pub async fn clear_inventory(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM scan_log").execute(pool).await?;
    sqlx::query("DELETE FROM inventory").execute(pool).await?;
    let _ = sqlx::query("DELETE FROM sqlite_sequence WHERE name IN ('inventory', 'scan_log')")
        .execute(pool)
        .await;
    Ok(())
}