use crate::types::Market;

/// Fee model based on Polymarket fee structure
#[derive(Debug, Clone)]
pub struct FeeModel {
    pub maker_fee_bps: u32, // Basis points (usually 0)
    pub taker_fee_bps: u32, // Basis points (usually ~200)
}

impl FeeModel {
    /// Create from market data
    #[allow(dead_code)]
    pub fn from_market(market: &Market) -> Self {
        Self {
            maker_fee_bps: market.maker_base_fee,
            taker_fee_bps: market.taker_base_fee,
        }
    }

    /// Calculate fee for a trade
    pub fn calculate(&self, notional: f64, is_maker: bool) -> f64 {
        let bps = if is_maker {
            self.maker_fee_bps
        } else {
            self.taker_fee_bps
        };
        notional * (bps as f64 / 10000.0)
    }

    /// Get taker fee as decimal
    #[allow(dead_code)]
    pub fn taker_rate(&self) -> f64 {
        self.taker_fee_bps as f64 / 10000.0
    }
}
