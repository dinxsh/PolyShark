use crate::types::{OrderBook, Side};

/// Fill rate estimator
#[derive(Debug, Clone)]
pub struct FillModel;

impl FillModel {
    /// Estimate how much of the order can fill
    pub fn estimate_fill_ratio(book: &OrderBook, size: f64, side: Side) -> f64 {
        let available = match side {
            Side::Buy => book.total_ask_liquidity(),
            Side::Sell => book.total_bid_liquidity(),
        };

        if available >= size {
            1.0
        } else {
            available / size
        }
    }

    /// Get filled size based on available liquidity
    pub fn filled_size(book: &OrderBook, requested_size: f64, side: Side) -> f64 {
        let ratio = Self::estimate_fill_ratio(book, requested_size, side);
        requested_size * ratio
    }
}
