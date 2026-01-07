use crate::types::Side;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
/// Represents the on-chain state of a MetaMask Smart Account (ERC-7715)
/// Tracks the "Daily Spend Limit" permission granted to this agent.
pub struct Wallet {
    pub daily_limit: f64,
    pub spent_today: f64,
    pub last_reset: u64,
    pub positions: HashMap<String, Position>,
    pub total_trades: u32,
    pub winning_trades: u32,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Position {
    pub token_id: String,
    pub side: Side,
    pub size: f64,
    pub entry_price: f64,
    pub entry_time: u64,
}

impl Wallet {
    /// Create new permissioned wallet adapter
    pub fn new(daily_limit: f64) -> Self {
        Self {
            daily_limit,
            spent_today: 0.0,
            last_reset: Self::current_timestamp(),
            positions: HashMap::new(),
            total_trades: 0,
            winning_trades: 0,
        }
    }

    pub fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    fn check_reset(&mut self) {
        let now = Self::current_timestamp();
        // Simple 24h reset logic
        if now - self.last_reset >= 86400 {
            self.spent_today = 0.0;
            self.last_reset = now;
            println!("🔄 [ERC-7715] Daily Limit Period Reset - Allowance Refreshed");
        }
    }

    /// Check if we have sufficient permission allowance
    pub fn check_permission(&mut self, amount: f64) -> bool {
        self.check_reset();
        (self.spent_today + amount) <= self.daily_limit
    }

    /// Record a spend against the permission
    pub fn record_spend(&mut self, amount: f64) -> bool {
        if self.check_permission(amount) {
            self.spent_today += amount;
            true
        } else {
            false
        }
    }

    /// Open a new position (tracking only)
    pub fn open_position(
        &mut self,
        token_id: String,
        side: Side,
        size: f64,
        price: f64,
        timestamp: u64,
    ) {
        self.positions.insert(
            token_id.clone(),
            Position {
                token_id,
                side,
                size,
                entry_price: price,
                entry_time: timestamp,
            },
        );
    }

    /// Close a position (tracking only)
    #[allow(dead_code)]
    pub fn close_position(&mut self, token_id: &str, _exit_price: f64) {
        self.positions.remove(token_id);
    }

    pub fn record_trade(&mut self, is_winner: bool) {
        self.total_trades += 1;
        if is_winner {
            self.winning_trades += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_daily_limits() {
        let mut wallet = Wallet::new(100.0);

        // Spend 50
        assert!(wallet.record_spend(50.0));
        assert_eq!(wallet.spent_today, 50.0);

        // Try spending 60 (should fail)
        assert!(!wallet.record_spend(60.0));
        assert_eq!(wallet.spent_today, 50.0);
    }
}
