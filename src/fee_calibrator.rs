#![allow(dead_code)]

pub struct FeeCalibrator;

impl FeeCalibrator {
    /// Calculate the 95th percentile fee rate from observed trades
    /// Logic: fee_rate = (expected_cost - actual_cost) / expected_cost
    /// But trades usually don't have "expected cost" fields, we derive from price * size vs total_paid?
    /// If we assume `Trade` struct has what we need.
    /// Actually context.md says: `fee_rate = (expected_cost - actual_cost) / expected_cost`
    /// We'll assume input is a list of inferred rates.
    pub fn calibration_fee_p95(rates: &[f64]) -> f64 {
        let mut sorted = rates.to_vec();
        // sort floats handling NaNs
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let len = sorted.len();
        if len == 0 {
            return 0.002;
        } // Default 2%

        let index = (len as f64 * 0.95) as usize;
        sorted[index.min(len - 1)]
    }

    /// Derive implied fee rate from a trade if we knew the raw price vs paid price
    /// This is a helper for the user to pipe data into.
    pub fn derive_rate(oracle_price: f64, execution_price: f64) -> f64 {
        // Simple diff model
        (execution_price - oracle_price).abs() / oracle_price
    }
}
