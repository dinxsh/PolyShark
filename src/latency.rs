use rand_distr::{Distribution, Normal};
use std::time::Duration;

/// Latency and adverse selection model
#[derive(Debug, Clone)]
pub struct LatencyModel {
    pub mean_delay_ms: u64,
    pub adverse_move_std: f64,
}

impl LatencyModel {
    pub fn new(mean_delay_ms: u64, adverse_move_std: f64) -> Self {
        Self {
            mean_delay_ms,
            adverse_move_std,
        }
    }

    /// Apply latency and adverse price movement
    pub fn apply(&self, signal_price: f64) -> (f64, Duration) {
        let delay = Duration::from_millis(self.mean_delay_ms);

        let mut rng = rand::thread_rng();
        // Fallback to simpler math if distribution creation fails, but Normal should work for std >= 0
        let move_pct = if self.adverse_move_std > 0.0 {
            let normal = Normal::new(0.0, self.adverse_move_std).unwrap();
            normal.sample(&mut rng)
        } else {
            0.0
        };

        let new_price = signal_price * (1.0 + move_pct);

        (new_price, delay)
    }
}
