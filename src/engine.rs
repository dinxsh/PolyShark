//! Trading Engine Module
//!
//! Orchestrates the main trading loop with safety controls and failure handling.

use crate::types::Side;
use crate::wallet::Wallet;
use crate::market::MarketDataProvider;
use crate::arb::ArbitrageDetector;
use crate::execution::ExecutionEngine;
use crate::config::SafetyConfig;
use std::time::{Duration, Instant};

/// Agent operational status for monitoring
#[derive(Debug, Clone, PartialEq)]
pub enum EngineStatus {
    /// Engine is running normally
    Running,
    /// Engine entered safe mode due to failures
    SafeMode { reason: String, until: Instant },
    /// Engine suspended due to data delay
    DataDelaySuspended { delay_ms: u64 },
    /// Engine stopped - permission expired or revoked
    Stopped,
}

#[allow(dead_code)]
pub struct TradingEngine {
    pub wallet: Wallet,
    pub market_provider: MarketDataProvider,
    pub detector: ArbitrageDetector,
    pub execution_engine: ExecutionEngine,
    /// Current engine status
    status: EngineStatus,
    /// Consecutive API failure count
    consecutive_failures: u32,
    /// Safety configuration
    safety_config: SafetyConfig,
    /// Last successful data fetch timestamp
    last_data_fetch: Option<Instant>,
}

impl TradingEngine {
    pub fn new(
        wallet: Wallet,
        market_provider: MarketDataProvider,
        detector: ArbitrageDetector,
        execution_engine: ExecutionEngine,
    ) -> Self {
        Self {
            wallet,
            market_provider,
            detector,
            execution_engine,
            status: EngineStatus::Running,
            consecutive_failures: 0,
            safety_config: SafetyConfig::default(),
            last_data_fetch: None,
        }
    }

    /// Create engine with custom safety configuration
    pub fn with_safety_config(mut self, config: SafetyConfig) -> Self {
        self.safety_config = config;
        self
    }

    /// Get current engine status
    pub fn get_status(&self) -> &EngineStatus {
        &self.status
    }

    /// Check if engine should enter safe mode
    /// 
    /// SAFETY: This is called before each tick to ensure we don't trade
    /// under dangerous conditions.
    fn check_safety_conditions(&mut self) -> bool {
        // Check if we're in safe mode cooldown
        if let EngineStatus::SafeMode { until, .. } = self.status {
            if Instant::now() < until {
                return false; // Still in cooldown
            }
            // Cooldown expired, try to resume
            println!("🔄 [Engine] Safe mode cooldown expired, attempting to resume...");
            self.status = EngineStatus::Running;
            self.consecutive_failures = 0;
        }

        // Check data staleness
        // FAILURE HANDLING: If data is stale (> max_data_delay_ms), suspend trading
        // to prevent trading on outdated market information.
        if let Some(last_fetch) = self.last_data_fetch {
            let delay = last_fetch.elapsed().as_millis() as u64;
            if delay > self.safety_config.max_data_delay_ms {
                println!("⚠️ [Engine] Data delay {}ms exceeds threshold {}ms - suspending",
                    delay, self.safety_config.max_data_delay_ms);
                self.status = EngineStatus::DataDelaySuspended { delay_ms: delay };
                return false;
            }
        }

        // Check consecutive failures
        // FAILURE HANDLING: If we have N consecutive API failures, enter safe mode
        // with a cooldown period to prevent hammering failing APIs.
        if self.consecutive_failures >= self.safety_config.max_consecutive_failures {
            let cooldown = Duration::from_secs(self.safety_config.safe_mode_cooldown_secs);
            println!("🛑 [Engine] {} consecutive failures - entering safe mode for {}s",
                self.consecutive_failures, cooldown.as_secs());
            self.status = EngineStatus::SafeMode {
                reason: format!("{} consecutive API failures", self.consecutive_failures),
                until: Instant::now() + cooldown,
            };
            return false;
        }

        true
    }

    /// Handle API failure with proper tracking
    /// 
    /// FAILURE HANDLING: Tracks consecutive failures and logs appropriately.
    fn handle_failure(&mut self, error: &dyn std::error::Error) {
        self.consecutive_failures += 1;
        println!("❌ [Engine] API failure #{}: {}", self.consecutive_failures, error);
    }

    /// Handle successful operation
    fn handle_success(&mut self) {
        self.consecutive_failures = 0;
        self.last_data_fetch = Some(Instant::now());
    }

    /// Run a single tick of the trading loop
    /// 
    /// SAFETY GUARANTEES:
    /// 1. Checks safety conditions before any trading
    /// 2. Tracks API failures and enters safe mode after threshold
    /// 3. Suspends on stale data
    /// 4. All errors are caught and handled gracefully
    pub async fn tick(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Pre-tick safety check
        if !self.check_safety_conditions() {
            return Ok(()); // Skip this tick, we're in a safety state
        }

        // Fetch markets with failure handling
        let markets = match self.market_provider.fetch_markets().await {
            Ok(m) => {
                self.handle_success();
                m
            }
            Err(e) => {
                self.handle_failure(&*e);
                return Err(e);
            }
        };

        // Scan for signals
        let signals = self.detector.scan(&markets);
        
        for signal in signals {
            // Simplified execution logic from main.rs
            if signal.recommended_side == Side::Buy {
               // Find market
               if let Some(market) = markets.iter().find(|m| m.id == signal.market_id) {
                    let size_per_leg = 5.0; // Fixed for now

                    // Execute on all outcomes (Buy Bundle behavior)
                    for token_id in &market.clob_token_ids {
                        match self.market_provider.fetch_order_book(token_id).await {
                            Ok(book) => {
                                self.execution_engine.execute(&book, size_per_leg, Side::Buy, &mut self.wallet);
                            }
                            Err(e) => {
                                // Log but don't fail entire tick for single order book fetch
                                println!("⚠️ [Engine] Order book fetch failed: {}", e);
                            }
                        }
                    }
               }
            }
        }
        Ok(())
    }

    /// Run the loop for a specific duration or number of ticks
    pub async fn run(&mut self, ticks: usize) {
        for tick_num in 0..ticks {
            if let Err(e) = self.tick().await {
                eprintln!("Error in tick {}: {}", tick_num, e);
            }
            // In simulation we might not want to sleep strictly, or sleep 0 for speed
            // simulating "ticks"
            tokio::time::sleep(Duration::from_millis(100)).await; 
        }
    }
}
