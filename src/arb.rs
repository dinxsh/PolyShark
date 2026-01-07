#![allow(dead_code)]
use crate::constraint::ConstraintChecker;
use crate::types::{ArbitrageSignal, Market};

/// Arbitrage detector
#[derive(Debug)]
pub struct ArbitrageDetector {
    pub constraint_checker: ConstraintChecker,
    pub min_profit_threshold: f64, // Minimum expected profit to trade
}

impl ArbitrageDetector {
    pub fn new(min_spread: f64, min_profit: f64) -> Self {
        Self {
            constraint_checker: ConstraintChecker::new(min_spread),
            min_profit_threshold: min_profit,
        }
    }

    /// Scan markets for arbitrage opportunities
    pub fn scan(&self, markets: &[Market]) -> Vec<ArbitrageSignal> {
        markets
            .iter()
            .filter(|m| m.active && m.accepting_orders)
            .filter_map(|m| self.constraint_checker.check_violation(m))
            .collect()
    }

    /// Calculate expected profit after costs
    pub fn expected_profit(
        &self,
        signal: &ArbitrageSignal,
        size: f64,
        fee_rate: f64,
        slippage: f64,
    ) -> f64 {
        let gross = signal.edge * size;
        let fee_cost = size * signal.yes_price * fee_rate * 2.0; // Both legs
        let slippage_cost = size * slippage;

        gross - fee_cost - slippage_cost
    }

    /// Decide if trade is worth taking
    pub fn should_trade(
        &self,
        signal: &ArbitrageSignal,
        size: f64,
        fee_rate: f64,
        slippage: f64,
    ) -> bool {
        self.expected_profit(signal, size, fee_rate, slippage) > self.min_profit_threshold
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Side;

    fn create_test_market(yes_price: f64, no_price: f64, active: bool) -> Market {
        Market {
            id: "test_market".to_string(),
            question: "Test question?".to_string(),
            slug: "test-market".to_string(),
            outcomes: vec!["Yes".to_string(), "No".to_string()],
            outcome_prices: vec![yes_price, no_price],
            clob_token_ids: vec!["token1".to_string(), "token2".to_string()],
            best_bid: Some(yes_price - 0.01),
            best_ask: Some(yes_price + 0.01),
            maker_base_fee: 0,
            taker_base_fee: 200,
            liquidity: 1000.0,
            volume_24hr: 5000.0,
            active,
            accepting_orders: true,
        }
    }

    #[test]
    fn test_scan_finds_arbitrage_opportunity() {
        let detector = ArbitrageDetector::new(0.02, 0.10);

        // Market with 5% spread (0.48 + 0.47 = 0.95, spread = 0.05)
        let market = create_test_market(0.48, 0.47, true);
        let signals = detector.scan(&[market]);

        assert_eq!(signals.len(), 1);
        assert!((signals[0].spread - 0.05).abs() < 0.001);
        assert_eq!(signals[0].recommended_side, Side::Buy);
    }

    #[test]
    fn test_scan_ignores_balanced_market() {
        let detector = ArbitrageDetector::new(0.02, 0.10);

        // Balanced market (0.50 + 0.50 = 1.0)
        let market = create_test_market(0.50, 0.50, true);
        let signals = detector.scan(&[market]);

        assert!(signals.is_empty());
    }

    #[test]
    fn test_scan_ignores_inactive_market() {
        let detector = ArbitrageDetector::new(0.02, 0.10);

        // Inactive market with arbitrage opportunity
        let market = create_test_market(0.48, 0.47, false);
        let signals = detector.scan(&[market]);

        assert!(signals.is_empty());
    }

    #[test]
    fn test_expected_profit_calculation() {
        let detector = ArbitrageDetector::new(0.02, 0.10);

        let signal = ArbitrageSignal {
            market_id: "test".to_string(),
            spread: 0.05,
            edge: 0.05,
            recommended_side: Side::Buy,
            yes_price: 0.48,
            no_price: 0.47,
        };

        // Size: 100, Fee: 2%, Slippage: 1%
        let profit = detector.expected_profit(&signal, 100.0, 0.02, 0.01);

        // gross = 0.05 * 100 = 5.0
        // fee_cost = 100 * 0.48 * 0.02 * 2 = 1.92
        // slippage_cost = 100 * 0.01 = 1.0
        // expected = 5.0 - 1.92 - 1.0 = 2.08
        assert!((profit - 2.08).abs() < 0.01);
    }

    #[test]
    fn test_should_trade_above_threshold() {
        let detector = ArbitrageDetector::new(0.02, 0.10);

        let signal = ArbitrageSignal {
            market_id: "test".to_string(),
            spread: 0.05,
            edge: 0.05,
            recommended_side: Side::Buy,
            yes_price: 0.48,
            no_price: 0.47,
        };

        // With these params, profit > 0.10, so should trade
        assert!(detector.should_trade(&signal, 100.0, 0.02, 0.01));
    }

    #[test]
    fn test_should_not_trade_below_threshold() {
        let detector = ArbitrageDetector::new(0.02, 5.0); // High threshold

        let signal = ArbitrageSignal {
            market_id: "test".to_string(),
            spread: 0.05,
            edge: 0.05,
            recommended_side: Side::Buy,
            yes_price: 0.48,
            no_price: 0.47,
        };

        // Expected profit ~2.08 < 5.0 threshold
        assert!(!detector.should_trade(&signal, 100.0, 0.02, 0.01));
    }
}
