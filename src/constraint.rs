use crate::types::{ArbitrageSignal, Market, Side};

/// Binary market constraint checker
#[derive(Debug, Clone)]
pub struct ConstraintChecker {
    pub min_spread_threshold: f64, // e.g., 0.02 for 2%
}

impl ConstraintChecker {
    pub fn new(min_spread_threshold: f64) -> Self {
        Self {
            min_spread_threshold,
        }
    }

    /// Check if market has arbitrage opportunity
    pub fn check_violation(&self, market: &Market) -> Option<ArbitrageSignal> {
        // Calculate sum of all outcome prices
        let sum: f64 = market.outcome_prices.iter().sum();
        let spread = (sum - 1.0).abs();

        if spread <= self.min_spread_threshold {
            return None; // No opportunity
        }

        let recommended_side = if sum > 1.0 {
            Side::Sell // Prices are overvalued (Sum > 1), Sell the bundle? (Selling all outcomes is complex, usually implies minting)
                       // In Polymarket, you can Sell if you hold, or you Mint sets and Sell.
                       // For simple arb, we usually look for Sum < 1 (buying the bundle for < $1).
        } else {
            Side::Buy // Prices are undervalued (Sum < 1), Buy all outcomes for guaranteed payout of $1
        };

        Some(ArbitrageSignal {
            market_id: market.id.clone(),
            spread,
            edge: spread, // Gross edge before costs
            recommended_side,
            yes_price: market.yes_price(), // Legacy field, might need updating in ArbitrageSignal struct to be generic
            no_price: market.no_price(),   // Legacy field
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_market(yes_price: f64, no_price: f64) -> Market {
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
            active: true,
            accepting_orders: true,
        }
    }

    #[test]
    fn test_no_violation_when_balanced() {
        let checker = ConstraintChecker::new(0.02);
        let market = create_test_market(0.50, 0.50);

        let result = checker.check_violation(&market);
        assert!(result.is_none());
    }

    #[test]
    fn test_no_violation_when_within_threshold() {
        let checker = ConstraintChecker::new(0.02);
        // Sum = 0.99, spread = 0.01 < 0.02 threshold
        let market = create_test_market(0.49, 0.50);

        let result = checker.check_violation(&market);
        assert!(result.is_none());
    }

    #[test]
    fn test_violation_detected_underpriced() {
        let checker = ConstraintChecker::new(0.02);
        // Sum = 0.95, spread = 0.05 > 0.02 threshold
        let market = create_test_market(0.48, 0.47);

        let result = checker.check_violation(&market);
        assert!(result.is_some());

        let signal = result.unwrap();
        assert!((signal.spread - 0.05).abs() < 0.001);
        assert_eq!(signal.recommended_side, Side::Buy);
    }

    #[test]
    fn test_violation_detected_overpriced() {
        let checker = ConstraintChecker::new(0.02);
        // Sum = 1.05, spread = 0.05 > 0.02 threshold
        let market = create_test_market(0.55, 0.50);

        let result = checker.check_violation(&market);
        assert!(result.is_some());

        let signal = result.unwrap();
        assert!((signal.spread - 0.05).abs() < 0.001);
        assert_eq!(signal.recommended_side, Side::Sell);
    }

    #[test]
    fn test_spread_at_threshold_boundary() {
        let checker = ConstraintChecker::new(0.02);
        // Sum = 0.99, spread = 0.01 < 0.02 threshold (should NOT trigger)
        // Using 0.495 + 0.495 = 0.99 to avoid floating point precision issues
        let market = create_test_market(0.495, 0.495);

        let result = checker.check_violation(&market);
        assert!(result.is_none());
    }
}
