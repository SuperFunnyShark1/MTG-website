use crate::db;
use reqwest::Client;
use serde::Deserialize;
use sqlx::SqlitePool;
use std::{
    collections::HashMap,
    env,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::RwLock;

const SCRYFALL_API_BASE: &str = "https://api.scryfall.com";
const FX_API_URL: &str = "https://open.er-api.com/v6/latest/USD";
const FX_CACHE_TTL: Duration = Duration::from_secs(6 * 60 * 60); // 6 hours

struct CachedRate {
    usd_to_clp: f64,
    fetched_at: Instant,
}

#[derive(Clone)]
pub struct ScryfallClient {
    http: Client,
    rate_cache: Arc<RwLock<Option<CachedRate>>>,
}

#[derive(Debug, Deserialize)]
struct ExchangeRateResponse {
    rates: HashMap<String, f64>,
}

#[derive(Debug, Deserialize)]
struct ScryfallCard {
    prices: Option<ScryfallPrices>,
    image_uris: Option<ScryfallImageUris>,
    card_faces: Option<Vec<ScryfallCardFace>>,
}

#[derive(Debug, Deserialize)]
struct ScryfallPrices {
    usd: Option<String>,
    usd_foil: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ScryfallImageUris {
    normal: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ScryfallCardFace {
    image_uris: Option<ScryfallImageUris>,
}

pub struct EnrichedCardData {
    pub price_usd: Option<f64>,
    pub price_clp: Option<i64>,
    pub image_url: Option<String>,
}

impl ScryfallClient {
    pub fn new() -> Self {
        let http = Client::builder()
            .user_agent("MTGStoreInventory/0.1 (CapstoneProject)")
            .timeout(Duration::from_secs(5))
            .build()
            .expect("Failed to build HTTP client");

        Self {
            http,
            rate_cache: Arc::new(RwLock::new(None)),
        }
    }

    /// Returns the cached USD -> CLP exchange rate, refreshing from open.er-api.com every 6 hours.
    pub async fn get_usd_to_clp_rate(&self) -> f64 {
        {
            let guard = self.rate_cache.read().await;
            if let Some(cached) = &*guard {
                if cached.fetched_at.elapsed() < FX_CACHE_TTL {
                    return cached.usd_to_clp;
                }
            }
        }

        if let Ok(resp) = self.http.get(FX_API_URL).send().await {
            if resp.status().is_success() {
                if let Ok(data) = resp.json::<ExchangeRateResponse>().await {
                    if let Some(&clp_rate) = data.rates.get("CLP") {
                        if clp_rate > 0.0 {
                            println!("Updated exchange rate: 1 USD = {:.2} CLP", clp_rate);
                            let mut guard = self.rate_cache.write().await;
                            *guard = Some(CachedRate {
                                usd_to_clp: clp_rate,
                                fetched_at: Instant::now(),
                            });
                            return clp_rate;
                        }
                    }
                }
            }
        }

        // Offline / API error fallback
        env::var("FALLBACK_USD_TO_CLP")
            .ok()
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(950.0)
    }

    pub async fn fetch_card_details(
        &self,
        set_code: &str,
        collector_number: &str,
        lang: &str,
        is_foil: bool,
    ) -> Result<EnrichedCardData, reqwest::Error> {
        let clean_num = match collector_number.trim().parse::<u32>() {
            Ok(n) => n.to_string(),
            Err(_) => collector_number.trim().to_string(),
        };
        let set = set_code.trim().to_lowercase();
        let lang_clean = lang.trim().to_lowercase();

        let localized_card = self.get_card_json(&set, &clean_num, Some(&lang_clean)).await?;

        let mut price_usd = localized_card
            .as_ref()
            .and_then(|c| Self::extract_price(c, is_foil));
        let mut image_url = localized_card
            .as_ref()
            .and_then(Self::extract_image_url);

        if (price_usd.is_none() || image_url.is_none()) && lang_clean != "en" {
            tokio::time::sleep(Duration::from_millis(100)).await;
            if let Some(en_card) = self.get_card_json(&set, &clean_num, None).await? {
                if price_usd.is_none() {
                    price_usd = Self::extract_price(&en_card, is_foil);
                }
                if image_url.is_none() {
                    image_url = Self::extract_image_url(&en_card);
                }
            }
        }

        let price_clp = match price_usd {
            Some(usd) => {
                let rate = self.get_usd_to_clp_rate().await;
                Some((usd * rate).round() as i64)
            }
            None => None,
        };

        Ok(EnrichedCardData {
            price_usd,
            price_clp,
            image_url,
        })
    }

    async fn get_card_json(
        &self,
        set_code: &str,
        collector_number: &str,
        lang: Option<&str>,
    ) -> Result<Option<ScryfallCard>, reqwest::Error> {
        let url = match lang {
            Some(l) if !l.is_empty() => {
                format!("{SCRYFALL_API_BASE}/cards/{set_code}/{collector_number}/{l}")
            }
            _ => format!("{SCRYFALL_API_BASE}/cards/{set_code}/{collector_number}"),
        };

        let resp = self.http.get(&url).send().await?;
        if resp.status().is_success() {
            let parsed = resp.json::<ScryfallCard>().await?;
            Ok(Some(parsed))
        } else {
            Ok(None)
        }
    }

    fn extract_price(card: &ScryfallCard, is_foil: bool) -> Option<f64> {
        let prices = card.prices.as_ref()?;
        let raw = if is_foil {
            prices.usd_foil.as_deref().or(prices.usd.as_deref())
        } else {
            prices.usd.as_deref()
        };
        raw.and_then(|s| s.parse::<f64>().ok())
    }

    fn extract_image_url(card: &ScryfallCard) -> Option<String> {
        if let Some(uris) = &card.image_uris {
            if let Some(url) = &uris.normal {
                return Some(url.clone());
            }
        }
        if let Some(faces) = &card.card_faces {
            if let Some(first) = faces.first() {
                if let Some(uris) = &first.image_uris {
                    return uris.normal.clone();
                }
            }
        }
        None
    }

    pub async fn refresh_all_prices(&self, pool: &SqlitePool) -> usize {
        let Ok(cards) = db::get_all_cards(pool).await else {
            return 0;
        };

        let mut updated_count = 0;
        for card in cards {
            if let Ok(details) = self
                .fetch_card_details(&card.set_code, &card.collector_number, &card.lang, card.is_foil)
                .await
            {
                if details.price_usd.is_some() || details.image_url.is_some() {
                    if db::update_card_enrichment(
                        pool,
                        card.id,
                        details.price_usd,
                        details.price_clp,
                        details.image_url.as_deref(),
                    )
                    .await
                    .is_ok()
                    {
                        updated_count += 1;
                    }
                }
            }
            tokio::time::sleep(Duration::from_millis(110)).await;
        }

        updated_count
    }
}