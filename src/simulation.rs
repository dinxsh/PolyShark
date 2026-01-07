use crate::arb::ArbitrageDetector;
use crate::engine::TradingEngine;
use crate::execution::ExecutionEngine;
use crate::fees::FeeModel;
use crate::latency::LatencyModel;
use crate::market::MarketDataProvider;
use crate::wallet::Wallet;
// use crate::types::Side; // Unused import

#[allow(dead_code)]
pub async fn run_monte_carlo(iterations: usize) {
    println!(
        "🎲 Starting Monte Carlo Simulation ({} runs)...",
        iterations
    );

    let mut total_pnl = 0.0;
    let mut wins = 0;
    let mut losses = 0;

    for i in 0..iterations {
        // Setup fresh environment for each run
        let wallet = Wallet::new(100.0); // Higher limit for sim
                                         // In a real MC, we'd vary these parameters randomly
        let fee_model = FeeModel {
            maker_fee_bps: 0,
            taker_fee_bps: 200,
        };
        let latency_model = LatencyModel::new(
            50 + (i as u64 % 50),     // Vary latency: 50-100ms
            0.001 * (i as f64 % 5.0), // Vary adverse move: 0% - 0.5%
        );

        let market_provider = MarketDataProvider::new("https://indexer.envio.dev/graphql");
        let detector = ArbitrageDetector::new(0.01, 0.05); // tighter spreads
        let execution_engine = ExecutionEngine::new(fee_model, latency_model);

        let mut engine = TradingEngine::new(wallet, market_provider, detector, execution_engine);

        // Run for 10 ticks
        engine.run(10).await;

        let pnl = engine.wallet.spent_today; // simplified "pnl" as "money deployed" for this demo
                                             // Real PnL requires closing positions which we haven't implemented logic for

        total_pnl += pnl;
        if pnl > 0.0 {
            wins += 1;
        } else {
            losses += 1;
        }

        if i % 10 == 0 {
            println!("Run {}: Deployed ${:.2}", i, pnl);
        }
    }

    println!("🏁 Simulation Complete!");
    println!("   Total Runs: {}", iterations);
    println!("   Total Volume: ${:.2}", total_pnl);
    println!("   Active runs: {} | Inactive runs: {}", wins, losses);
}
