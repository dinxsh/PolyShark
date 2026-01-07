use crate::types::{OrderBook, Side};

/// Slippage calculator using order book
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SlippageModel;

#[allow(dead_code)]
impl SlippageModel {
    /// Calculate slippage from order book
    pub fn calculate(book: &OrderBook, size: f64, side: Side) -> Option<f64> {
        let midpoint = book.midpoint()?;
        let exec_price = book.execution_price(size, side)?;

        let slippage = match side {
            Side::Buy => (exec_price - midpoint) / midpoint,
            Side::Sell => (midpoint - exec_price) / midpoint,
        };

        Some(slippage)
    }

    /// Estimate execution cost including slippage
    pub fn execution_cost(book: &OrderBook, size: f64, side: Side) -> Option<f64> {
        let exec_price = book.execution_price(size, side)?;
        Some(exec_price * size)
    }
}
