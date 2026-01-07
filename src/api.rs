//! HTTP API module for Frontend <-> Agent communication
//!
//! Exposes endpoints for the dashboard to control the agent and view stats.

use crate::metamask::{MetaMaskClient, PermissionGrant};
use crate::positions::PositionManager;
use crate::types::Market;
use serde::Serialize;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use warp::Filter;

/// Cached market data with timestamp
#[derive(Clone)]
pub struct MarketCache {
    pub markets: Vec<Market>,
    pub last_update: Option<Instant>,
    pub signal_count: usize,
}

impl Default for MarketCache {
    fn default() -> Self {
        Self {
            markets: Vec::new(),
            last_update: None,
            signal_count: 0,
        }
    }
}

/// API Server State
#[derive(Clone)]
pub struct ApiState {
    pub metamask: Arc<MetaMaskClient>,
    pub position_manager: Arc<RwLock<PositionManager>>,
    pub market_cache: Arc<RwLock<MarketCache>>,
}

/// Start the API server
pub async fn start_server(state: ApiState) {
    // CORS configuration
    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec!["content-type"])
        .allow_methods(vec!["GET", "POST", "OPTIONS"]);

    // POST /api/permission
    // Receives permission grant from frontend (MetaMask)
    let permission_route = warp::path!("api" / "permission")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_state(state.clone()))
        .and_then(handle_permission);

    // GET /api/stats
    // Returns live stats for dashboard
    let stats_route = warp::path!("api" / "stats")
        .and(warp::get())
        .and(with_state(state.clone()))
        .and_then(handle_stats);

    // GET /api/markets
    // Returns cached market data for dashboard
    let markets_route = warp::path!("api" / "markets")
        .and(warp::get())
        .and(with_state(state.clone()))
        .and_then(handle_markets);

    // Serve dashboard static files
    // Get the dashboard directory path (relative to executable or use manifest dir for dev)
    let dashboard_dir = get_dashboard_path();
    println!("📂 [API] Serving dashboard from: {:?}", dashboard_dir);

    // Serve index.html at root path
    let index_route = warp::path::end()
        .and(warp::get())
        .and(warp::fs::file(dashboard_dir.join("index.html")));

    // Serve other static files from dashboard directory
    let static_route = warp::fs::dir(dashboard_dir);

    let routes = permission_route
        .or(stats_route)
        .or(markets_route)
        .or(index_route)
        .or(static_route)
        .with(cors);

    println!("🌍 [API] Server starting on http://localhost:3030");
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

fn with_state(
    state: ApiState,
) -> impl Filter<Extract = (ApiState,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || state.clone())
}

/// Handle permission update from frontend
async fn handle_permission(
    grant: PermissionGrant, // Frontend sends the grant object directly
    state: ApiState,
) -> Result<impl warp::Reply, warp::Rejection> {
    println!(
        "📥 [API] Received permission grant from Dashboard: {}",
        grant.permission_id
    );

    // Update the MetaMask client
    state.metamask.set_permission(grant).await;

    Ok(warp::reply::json(&serde_json::json!({ "status": "ok" })))
}

#[derive(Serialize)]
struct StatsResponse {
    connected: bool, // Agent is running
    permission_active: bool,
    daily_limit: f64,
    spent_today: f64,
    total_trades: usize,
    win_rate: f64,
    total_pnl: f64,
    open_positions: usize,
}

/// Handle stats request
async fn handle_stats(state: ApiState) -> Result<impl warp::Reply, warp::Rejection> {
    let perm = state.metamask.get_permission().await;
    let pm = state.position_manager.read().await;

    let (active, limit, spent) = match perm {
        Some(p) => (!p.revoked, p.daily_limit, p.spent_today),
        None => (false, 0.0, 0.0),
    };

    let stats = StatsResponse {
        connected: true,
        permission_active: active,
        daily_limit: limit,
        spent_today: spent,
        total_trades: pm.trade_count(),
        win_rate: pm.win_rate() * 100.0,
        total_pnl: pm.total_pnl(),
        open_positions: pm.get_positions().len(),
    };

    Ok(warp::reply::json(&stats))
}

/// Market info for API response
#[derive(Serialize)]
struct MarketInfo {
    id: String,
    question: String,
    slug: String,
    outcomes: Vec<String>,
    prices: Vec<f64>,
    active: bool,
}

/// Markets API response
#[derive(Serialize)]
struct MarketsResponse {
    markets: Vec<MarketInfo>,
    total_count: usize,
    last_update_ms: u64,
    signal_count: usize,
}

/// Handle markets request
async fn handle_markets(state: ApiState) -> Result<impl warp::Reply, warp::Rejection> {
    let cache = state.market_cache.read().await;

    let last_update_ms = cache
        .last_update
        .map(|t| t.elapsed().as_millis() as u64)
        .unwrap_or(0);

    let markets: Vec<MarketInfo> = cache
        .markets
        .iter()
        .take(20)
        .map(|m| MarketInfo {
            id: m.id.clone(),
            question: m.question.clone(),
            slug: m.slug.clone(),
            outcomes: m.outcomes.clone(),
            prices: m.outcome_prices.clone(),
            active: m.active,
        })
        .collect();

    let response = MarketsResponse {
        total_count: cache.markets.len(),
        markets,
        last_update_ms,
        signal_count: cache.signal_count,
    };

    Ok(warp::reply::json(&response))
}

/// Get the path to the dashboard directory
/// Uses CARGO_MANIFEST_DIR during development, falls back to current directory
fn get_dashboard_path() -> PathBuf {
    // In development, use the manifest directory
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        return PathBuf::from(manifest_dir).join("dashboard");
    }

    // In production, look for dashboard next to the executable
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let dashboard_path = exe_dir.join("dashboard");
            if dashboard_path.exists() {
                return dashboard_path;
            }
        }
    }

    // Fallback to current directory
    PathBuf::from("dashboard")
}
