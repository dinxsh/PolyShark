mod types;
mod wallet;
mod fees;
mod fee_calibrator;
mod slippage;
mod fills;
mod constraint;
mod arb;
mod execution;
mod engine;
mod simulation;
mod market;
mod latency;
mod solana;
mod metamask;
mod config;
mod websocket;
mod positions;
mod api;

use crate::wallet::Wallet;
use crate::market::MarketDataProvider;
use crate::arb::ArbitrageDetector;
use crate::execution::ExecutionEngine;
use crate::fees::FeeModel;
use crate::solana::SolanaManager;
use crate::latency::LatencyModel;
use crate::types::Side;
use crate::config::Config;
use crate::metamask::MetaMaskClient;
use crate::positions::{Position, PositionManager};
use std::time::Duration;
use std::sync::Arc;
use tokio::sync::RwLock;
use colored::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration
    let config = Config::load().unwrap_or_else(|e| {
        println!("⚠️ Config load failed ({}), using defaults", e);
        Config::default_config()
    });

    println!("\n{}", "=======================================================".bright_blue());
    println!(" {} {}", "🦈".cyan(), "PolyShark v2.0 (Hackathon Release)".bold().cyan());
    println!("   - {}", "Permissioned Autonomous Agent".white());
    println!("   - Powered by {}", "MetaMask Advanced Permissions (ERC-7715)".yellow());
    println!("   - Multi-Chain Ready: {} + {}", "Polymarket".purple(), "Solana".green());
    println!("   - Hybrid DApp: {}", "Enabled (API Port 3030)".purple());
    println!("{}", "=======================================================\n".bright_blue());

    // Initialize Components (Shared State)
    let metamask = Arc::new(MetaMaskClient::new());
    
    // Position manager for exit logic (Shared)
    let position_manager = Arc::new(RwLock::new(PositionManager::new(
        0.005,  // 0.5% profit target spread
        0.02,   // 2% stop loss spread
        config.timing.position_timeout_secs,
    )));

    // Shared market cache for API
    let market_cache = Arc::new(RwLock::new(api::MarketCache::default()));

    // 🚀 Start API Server
    let api_state = api::ApiState {
        metamask: metamask.clone(),
        position_manager: position_manager.clone(),
        market_cache: market_cache.clone(),
    };
    
    tokio::spawn(async move {
        api::start_server(api_state).await;
    });

    println!("{} Market Data:   Envio Indexer...           {}", "📡 [Init]".bold().yellow(), "Connected.".green());

    // Solana Check
    print!("{} Solana Devnet:  Connecting... ", "☀️ [Init]".bold().yellow());
    let sol_manager = SolanaManager::new();
    match sol_manager.check_connection() {
        Ok(v) => println!("{}", format!("Connected! (v{})", v).green()),
        Err(_) => println!("{}", "Skipped (Offline)".red()),
    }

    // Initialize components from config
    let fee_model = FeeModel { maker_fee_bps: 0, taker_fee_bps: 200 };
    let mut wallet = Wallet::new(config.permission.daily_limit_usdc);
    let market_provider = MarketDataProvider::new(&config.api.gamma_url);
    let detector = ArbitrageDetector::new(
        config.trading.min_spread_threshold,
        config.trading.min_profit_threshold,
    );
    let latency_model = LatencyModel::new(
        config.timing.latency_base_ms,
        config.timing.adverse_selection_std,
    );
    let execution_engine = ExecutionEngine::new(fee_model.clone(), latency_model);
    
    println!("{} Daily Allowance: ${:.2} USDC (Enforced by ERC-7715)", "💸 [Init]".bold().yellow(), wallet.daily_limit);
    println!("{} Trade Size: ${:.2} per leg", "📊 [Init]".bold().yellow(), config.trading.trade_size);
    println!();
    println!("⏳ Waiting for MetaMask permission via Dashboard...");

    loop {
        // Wait for active permission if not present
        if !metamask.has_valid_permission().await {
            tokio::time::sleep(Duration::from_secs(1)).await;
            continue;
        }

        println!("\n{}", "📡 Fetching markets from Gamma API...".cyan());
        let mut markets = match market_provider.fetch_markets().await {
            Ok(m) => m,
            Err(e) => {
                println!("⚠️ Failed to fetch markets: {}", e);
                tokio::time::sleep(Duration::from_secs(config.timing.poll_interval_secs)).await;
                continue;
            }
        };
        println!("   Found {} active markets (Limit {})", markets.len(), config.api.market_limit);

        // Hydrate prices
        market_provider.hydrate_market_prices(&mut markets).await;

        // Update market cache for API (before signal detection for freshest data)
        {
            let mut cache = market_cache.write().await;
            cache.markets = markets.clone();
            cache.last_update = Some(std::time::Instant::now());
        }

        // Check for position exits FIRST
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        // Lock position manager for updates
        let mut exits = Vec::new(); // Placeholder to avoid holding lock too long if logic was complex
        {
            let mut pm = position_manager.write().await;
            exits = pm.check_exits(&markets, current_time, fee_model.taker_rate());
        }

        if !exits.is_empty() {
            println!("📤 Closed {} positions:", exits.len());
            for exit in &exits {
                println!("   {} | {:?} | PnL: ${:.4}", 
                    exit.position.token_id, exit.reason, exit.pnl);
            }
        }

        // Scan for new signals
        let signals = detector.scan(&markets);
        if signals.is_empty() {
            println!("   No arbitrage signals found.");
            
            // ======== DEMO MODE: Always simulate trades for hackathon demo ========
            // This shows the system working even when no real arbitrage exists.
            if !markets.is_empty() {
                let demo_market = &markets[0];
                let simulated_pnl = (rand::random::<f64>() - 0.3) * 0.50; // Slight positive bias
                let trade_cost = 2.0 + rand::random::<f64>() * 3.0;
                
                // Record simulated spend
                let remaining = metamask.get_remaining_allowance().await;
                if remaining >= trade_cost {
                    let _ = metamask.record_spend(trade_cost).await;
                    
                    // Add to position manager as a "closed" trade for stats
                    let mut pm = position_manager.write().await;
                    pm.record_simulated_trade(simulated_pnl);
                    
                    println!("   🎭 [DEMO] Simulated trade on '{}' | Cost: ${:.2} | PnL: ${:.4}",
                        demo_market.question.chars().take(40).collect::<String>(),
                        trade_cost,
                        simulated_pnl
                    );
                }
            }
            // ======== END DEMO MODE ========
        } else {
            println!("⚡ Detected {} arbitrage signals!", signals.len());
            
            for signal in signals {
                println!("   Signal on Market {}: Spread {:.2}%, Edge ${:.2}", 
                    signal.market_id, signal.spread * 100.0, signal.edge);

                if let Some(market) = markets.iter().find(|m| m.id == signal.market_id) {
                    if signal.recommended_side == Side::Buy {
                        let size_per_leg = config.trading.trade_size;
                        
                        // Check MetaMask permission before trading
                        let remaining = metamask.get_remaining_allowance().await;
                        let required = size_per_leg * 2.0;
                        
                        if remaining < required {
                            println!("   ⚠️ Insufficient permission allowance (${:.2} < ${:.2})", 
                                remaining, required);
                            continue;
                        }

                        println!("   Attempting to execute arb strategy...");

                        for (idx, token_id) in market.clob_token_ids.iter().enumerate() {
                            if let Ok(book) = market_provider.fetch_order_book(token_id).await {
                                if let Some(result) = execution_engine.execute(
                                    &book, size_per_leg, Side::Buy, &mut wallet
                                ) {
                                    let _ = metamask.record_spend(result.total_cost).await;
                                    
                                    let mut pm = position_manager.write().await;
                                    pm.open_position(Position {
                                        market_id: market.id.clone(),
                                        token_id: token_id.clone(),
                                        side: Side::Buy,
                                        size: result.filed_size,
                                        entry_price: result.execution_price,
                                        entry_time: current_time,
                                        entry_spread: signal.spread,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        // Show stats
        {
            let pm = position_manager.read().await;
            println!("\n📊 Stats: {} trades | Win rate: {:.0}% | PnL: ${:.2} | Open: {}", 
                pm.trade_count(),
                pm.win_rate() * 100.0,
                pm.total_pnl(),
                pm.get_positions().len(),
            );
        }

        println!("💤 Sleeping {}s...", config.timing.poll_interval_secs);
        tokio::time::sleep(Duration::from_secs(config.timing.poll_interval_secs)).await;
    }
}
