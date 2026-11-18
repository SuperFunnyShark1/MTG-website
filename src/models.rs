use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Deserialize)]
pub struct IncomingScan {
    pub set_code: String,
    pub collector_number: String,
    #[serde(default = "default_lang")]
    pub lang: String,
    #[serde(default)]
    pub is_foil: bool,
    pub name: String,
    pub name_en: Option<String>,
    pub name_es: Option<String>,
    pub rarity: Option<String>,
    #[serde(default)]
    pub colors: Vec<String>,
    pub image_url: Option<String>,
    pub bin_number: Option<i64>,
    #[serde(default = "default_source")]
    pub source: String,
}

fn default_lang() -> String {
    "en".to_string()
}

fn default_source() -> String {
    "unknown".to_string()
}

#[derive(Debug, Serialize, FromRow)]
pub struct CardStockItem {
    pub id: i64,
    pub set_code: String,
    pub collector_number: String,
    pub lang: String,
    pub is_foil: bool,
    pub name: String,
    pub name_en: Option<String>,
    pub name_es: Option<String>,
    pub rarity: Option<String>,
    pub colors: String,
    pub price_usd: Option<f64>,
    pub price_clp: Option<i64>,
    pub image_url: Option<String>,
    pub bin_number: Option<i64>,
    pub quantity: i64,
    pub last_updated: String,
}