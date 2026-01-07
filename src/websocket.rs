//! WebSocket streaming module for real-time price updates
//!
//! Connects to Polymarket's WebSocket API for low-latency price feeds.

use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tokio_tungstenite::{connect_async, tungstenite::Message};

/// WebSocket message types from Polymarket
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WsMessage {
    #[serde(rename = "price_update")]
    PriceUpdate {
        market_id: String,
        token_id: String,
        price: f64,
        timestamp: u64,
    },
    #[serde(rename = "trade")]
    Trade {
        market_id: String,
        price: f64,
        size: f64,
        side: String,
        timestamp: u64,
    },
    #[serde(rename = "book_update")]
    BookUpdate { market_id: String, timestamp: u64 },
    #[serde(other)]
    Unknown,
}

/// Subscription request
#[allow(dead_code)]
#[derive(Debug, Serialize)]
struct SubscribeRequest {
    #[serde(rename = "type")]
    msg_type: String,
    channel: String,
    markets: Vec<String>,
}

/// WebSocket connection status
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum WsStatus {
    Disconnected,
    Connecting,
    Connected,
    Reconnecting,
    Failed(String),
}

/// Price cache updated by WebSocket
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct PriceCache {
    /// Map of token_id -> latest price
    pub prices: std::collections::HashMap<String, f64>,
    /// Last update timestamp
    pub last_update: u64,
}

/// WebSocket client for real-time Polymarket data
#[allow(dead_code)]
pub struct WebSocketClient {
    url: String,
    status: Arc<RwLock<WsStatus>>,
    price_cache: Arc<RwLock<PriceCache>>,
    /// Broadcast channel for price updates
    tx: broadcast::Sender<WsMessage>,
}

impl WebSocketClient {
    #[allow(dead_code)]
    pub fn new(url: &str) -> Self {
        let (tx, _) = broadcast::channel(1000);
        Self {
            url: url.to_string(),
            status: Arc::new(RwLock::new(WsStatus::Disconnected)),
            price_cache: Arc::new(RwLock::new(PriceCache::default())),
            tx,
        }
    }

    /// Get current connection status
    #[allow(dead_code)]
    pub async fn get_status(&self) -> WsStatus {
        self.status.read().await.clone()
    }

    /// Get a receiver for price updates
    #[allow(dead_code)]
    pub fn subscribe(&self) -> broadcast::Receiver<WsMessage> {
        self.tx.subscribe()
    }

    /// Get current price from cache
    #[allow(dead_code)]
    pub async fn get_price(&self, token_id: &str) -> Option<f64> {
        self.price_cache.read().await.prices.get(token_id).copied()
    }

    /// Connect and start streaming
    #[allow(dead_code)]
    pub async fn connect(&self, market_ids: Vec<String>) -> Result<(), WsError> {
        *self.status.write().await = WsStatus::Connecting;

        println!(
            "📡 [WebSocket] Connecting to {}...",
            &self.url[..50.min(self.url.len())]
        );

        let (ws_stream, _) = connect_async(&self.url)
            .await
            .map_err(|e| WsError::ConnectionFailed(e.to_string()))?;

        let (mut write, mut read) = ws_stream.split();

        *self.status.write().await = WsStatus::Connected;
        println!("✅ [WebSocket] Connected!");

        // Subscribe to markets
        let subscribe_msg = SubscribeRequest {
            msg_type: "subscribe".to_string(),
            channel: "market".to_string(),
            markets: market_ids,
        };

        let msg = serde_json::to_string(&subscribe_msg)
            .map_err(|e| WsError::SerializeError(e.to_string()))?;

        write
            .send(Message::Text(msg.into()))
            .await
            .map_err(|e| WsError::SendError(e.to_string()))?;

        println!("📝 [WebSocket] Subscribed to market channel");

        // Start reading messages
        let tx = self.tx.clone();
        let price_cache = self.price_cache.clone();
        let status = self.status.clone();

        tokio::spawn(async move {
            while let Some(msg) = read.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        if let Ok(ws_msg) = serde_json::from_str::<WsMessage>(&text) {
                            // Update cache
                            if let WsMessage::PriceUpdate {
                                ref token_id,
                                price,
                                timestamp,
                                ..
                            } = ws_msg
                            {
                                let mut cache = price_cache.write().await;
                                cache.prices.insert(token_id.clone(), price);
                                cache.last_update = timestamp;
                            }

                            // Broadcast to subscribers
                            let _ = tx.send(ws_msg);
                        }
                    }
                    Ok(Message::Close(_)) => {
                        *status.write().await = WsStatus::Disconnected;
                        println!("📴 [WebSocket] Connection closed");
                        break;
                    }
                    Err(e) => {
                        *status.write().await = WsStatus::Failed(e.to_string());
                        println!("❌ [WebSocket] Error: {}", e);
                        break;
                    }
                    _ => {}
                }
            }
        });

        Ok(())
    }
}

#[derive(Debug)]
pub enum WsError {
    ConnectionFailed(String),
    SerializeError(String),
    SendError(String),
}

impl std::fmt::Display for WsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ConnectionFailed(e) => write!(f, "WebSocket connection failed: {}", e),
            Self::SerializeError(e) => write!(f, "Serialization error: {}", e),
            Self::SendError(e) => write!(f, "Send error: {}", e),
        }
    }
}

impl std::error::Error for WsError {}
