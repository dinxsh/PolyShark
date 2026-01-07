use crate::fees::FeeModel;
use crate::fills::FillModel;
use crate::latency::LatencyModel;
use crate::types::{ExecutionResult, OrderBook, Side};
use crate::wallet::Wallet;
use std::thread;

/// Execution simulator
#[derive(Debug)]
pub struct ExecutionEngine {
    pub fee_model: FeeModel,
    pub latency_model: LatencyModel,
}

impl ExecutionEngine {
    pub fn new(fee_model: FeeModel, latency_model: LatencyModel) -> Self {
        Self {
            fee_model,
            latency_model,
        }
    }

    /// Simulate order execution
    pub fn execute(
        &self,
        book: &OrderBook,
        size: f64,
        side: Side,
        wallet: &mut Wallet,
    ) -> Option<ExecutionResult> {
        // 1. Calculate initial theoretical price
        let initial_price = book.execution_price(size, side)?;

        // 2. Apply latency and adverse selection
        let (exec_price, delay) = self.latency_model.apply(initial_price);

        // Simulate the delay
        if !delay.is_zero() {
            thread::sleep(delay);
        }

        // 3. Check fill ratio
        let filled_size = FillModel::filled_size(book, size, side);
        if filled_size <= 0.0 {
            return None;
        }

        // 4. Calculate execution metrics
        let midpoint = book.midpoint().unwrap_or(exec_price);
        let slippage = ((exec_price - midpoint) / midpoint).abs();

        // 5. Calculate costs
        let notional = exec_price * filled_size;
        let fee = self.fee_model.calculate(notional, false); // Taker
        let total_cost = notional + fee;

        // 6. Check permission (ERC-7715)
        if !wallet.check_permission(total_cost) {
            let remaining = wallet.daily_limit - wallet.spent_today;
            println!("❌ [Smart Account] Permission Denied: Trade value ${:.2} exceeds remaining Daily Allowance (${:.2})", 
                total_cost, remaining);
            return None;
        }

        // 7. Execute via Smart Account
        if wallet.record_spend(total_cost) {
            let remaining = wallet.daily_limit - wallet.spent_today;
            println!(
                "✅ [Smart Account] Batch Executed: Swap {:.2} USDC -> Tokens",
                total_cost
            );
            println!(
                "   ↳ Cost: ${:.2} | Latency: {:?} | Remaining Allowance: ${:.2}",
                total_cost, delay, remaining
            );

            // Track position
            let token_id = &book.token_id;
            wallet.open_position(
                token_id.clone(),
                side,
                filled_size,
                exec_price,
                crate::wallet::Wallet::current_timestamp(),
            );

            wallet.record_trade(true);

            Some(ExecutionResult {
                filled_size: filled_size,
                execution_price: exec_price,
                fee_paid: fee,
                slippage,
                total_cost,
                success: true,
            })
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::latency::LatencyModel;
    use crate::types::{OrderBook, PriceLevel};

    #[test]
    fn test_execution_permission_logic() {
        let fee_model = FeeModel {
            maker_fee_bps: 0,
            taker_fee_bps: 0,
        };
        let latency_model = LatencyModel::new(0, 0.0);
        let engine = ExecutionEngine::new(fee_model, latency_model);

        let mut wallet = Wallet::new(10.0);
        let book = OrderBook {
            token_id: "t1".to_string(),
            bids: vec![],
            asks: vec![PriceLevel {
                price: 0.5,
                size: 100.0,
            }],
            timestamp: 0,
        };

        // 1. Valid trade ($5 cost)
        let res = engine.execute(&book, 10.0, Side::Buy, &mut wallet);
        assert!(res.is_some());
        assert_eq!(wallet.spent_today, 5.0);

        // 2. Invalid trade ($6 cost, remaining limit $5)
        let res_fail = engine.execute(&book, 12.0, Side::Buy, &mut wallet);
        assert!(res_fail.is_none());
        assert_eq!(wallet.spent_today, 5.0);
    }
}
