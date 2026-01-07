use crate::types::{Market, OrderBook, PriceLevel};
use serde_json::Value;
use std::error::Error;

#[allow(dead_code)]
pub struct MarketDataProvider {
    client: reqwest::Client,
    gamma_url: String,
    clob_url: String,
}

impl MarketDataProvider {
    pub fn new(_envio_url: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            gamma_url: "https://gamma-api.polymarket.com/events?limit=20&active=true&closed=false"
                .to_string(),
            clob_url: "https://clob.polymarket.com/book".to_string(),
        }
    }

    /// Fetch all active markets from Gamma API
    pub async fn fetch_markets(&self) -> Result<Vec<Market>, Box<dyn Error>> {
        println!("🌐 Fetching LIVE market data from Gamma API...");
        let resp = self
            .client
            .get(&self.gamma_url)
            .send()
            .await?
            .text()
            .await?;
        let json: Value = serde_json::from_str(&resp)?;

        let mut markets = Vec::new();

        if let Some(events) = json.as_array() {
            for event in events {
                if let Some(event_markets) = event["markets"].as_array() {
                    for m in event_markets {
                        // Extract basic fields
                        let id = m["id"].as_str().unwrap_or("").to_string();
                        let question = m["question"].as_str().unwrap_or("").to_string();
                        let slug = event["slug"].as_str().unwrap_or("").to_string();

                        // Extract outcomes
                        let outcomes: Vec<String> = m["outcomes"]
                            .as_array()
                            .map(|arr| {
                                arr.iter()
                                    .map(|v| v.as_str().unwrap_or("").to_string())
                                    .collect()
                            })
                            .unwrap_or_default();

                        // Extract CLOB Token IDs (Critical)
                        // Note: Gamma API returns this as a STRINGIFIED JSON array, e.g. "[\"123\", \"456\"]"
                        let clob_token_ids: Vec<String> =
                            if let Some(s) = m["clobTokenIds"].as_str() {
                                serde_json::from_str(s).unwrap_or_default()
                            } else {
                                // Fallback if it somehow is an actual array (future proofing)
                                m["clobTokenIds"]
                                    .as_array()
                                    .map(|arr| {
                                        arr.iter()
                                            .map(|v| v.as_str().unwrap_or("").to_string())
                                            .collect()
                                    })
                                    .unwrap_or_default()
                            };

                        // Debug: Print what we found
                        // println!("DEBUG: Found market '{}' with {} tokens", slug, clob_token_ids.len());

                        // Skip if incomplete execution data
                        if clob_token_ids.len() < 2 {
                            // println!("DEBUG: Skipping {} (Not enough tokens)", slug);
                            continue;
                        }

                        markets.push(Market {
                            id,
                            question,
                            slug,
                            outcomes,
                            outcome_prices: vec![0.5, 0.5], // Will be updated by book fetch
                            clob_token_ids,
                            best_bid: None,
                            best_ask: None,
                            maker_base_fee: 0,
                            taker_base_fee: 200, // Standard 2%
                            liquidity: 0.0,      // Updated lazily
                            volume_24hr: 0.0,
                            active: true,
                            accepting_orders: true,
                        });
                    }
                }
            }
        }

        Ok(markets)
    }

    /// concurrently hydrate prices for all markets (Batch/Parallel)
    pub async fn hydrate_market_prices(&self, markets: &mut Vec<Market>) {
        use futures_util::stream::{self, StreamExt};

        println!("⚡ Hydrating prices concurrently (Concurrency: 50)...");
        let start = std::time::Instant::now();

        // 1. Flatten all tasks: (market_idx, token_idx, token_id)
        let mut tasks = Vec::new();
        for (m_idx, market) in markets.iter().enumerate() {
            for (t_idx, token_id) in market.clob_token_ids.iter().enumerate() {
                tasks.push((m_idx, t_idx, token_id.clone()));
            }
        }

        // 2. Create stream
        let fetches = stream::iter(tasks)
            .map(|(m_idx, t_idx, token_id)| {
                let client = &self; // Ref to self
                async move {
                    let res = client.fetch_order_book(&token_id).await;
                    (m_idx, t_idx, res)
                }
            })
            .buffer_unordered(50); // Concurrent limit

        // 3. Collect results
        let results: Vec<_> = fetches.collect().await;

        // 4. Update markets
        let mut update_count = 0;
        for (m_idx, t_idx, res) in results {
            if let Ok(book) = res {
                let price = book.midpoint().unwrap_or(0.0);
                if price > 0.0 {
                    // println!("   CTX: Market {} | Token {} | Price: {:.3}", markets[m_idx].slug, t_idx, price);
                    // Ensure vector is sized (it should be 2, but let's be safe)
                    if t_idx < markets[m_idx].outcome_prices.len() {
                        markets[m_idx].outcome_prices[t_idx] = price;
                        update_count += 1;
                    }
                }
            }
        }

        println!(
            "   ✅ Updated {} prices in {:.2?}",
            update_count,
            start.elapsed()
        );
    }

    /// Fetch order book for a market from CLOB API
    pub async fn fetch_order_book(&self, token_id: &str) -> Result<OrderBook, Box<dyn Error>> {
        let url = format!("{}?token_id={}", self.clob_url, token_id);
        let resp = self.client.get(&url).send().await?.text().await?;
        let json: Value = serde_json::from_str(&resp)?;

        // Helper to parse price/size strings
        let parse_level = |level: &Value| -> Option<PriceLevel> {
            let p = level["price"].as_str()?.parse::<f64>().ok()?;
            let s = level["size"].as_str()?.parse::<f64>().ok()?;
            Some(PriceLevel { price: p, size: s })
        };

        let bids: Vec<PriceLevel> = json["bids"]
            .as_array()
            .map(|arr| arr.iter().filter_map(parse_level).collect())
            .unwrap_or_default();

        let asks: Vec<PriceLevel> = json["asks"]
            .as_array()
            .map(|arr| arr.iter().filter_map(parse_level).collect())
            .unwrap_or_default();

        Ok(OrderBook {
            token_id: token_id.to_string(),
            bids,
            asks,
            timestamp: 0, // Not provided by snapshot endpoint cleanly
        })
    }
}
