//! Position management module
//! 
//! Handles position tracking, mean reversion exits, and PnL calculation.

use crate::types::{Market, Side};
use std::collections::HashMap;

/// An open position in the market
#[derive(Debug, Clone)]
pub struct Position {
    pub market_id: String,
    pub token_id: String,
    pub side: Side,
    pub size: f64,
    pub entry_price: f64,
    pub entry_time: u64,
    pub entry_spread: f64,  // Spread at entry for mean reversion tracking
}

/// Position exit reason
#[derive(Debug, Clone)]
pub enum ExitReason {
    MeanReversion,      // Spread normalized
    #[allow(dead_code)]
    ProfitTarget,       // Hit profit target
    StopLoss,           // Hit stop loss
    Timeout,            // Position held too long
    #[allow(dead_code)]
    Manual,             // Manual close
}

/// Position exit result
#[derive(Debug, Clone)]
pub struct ExitResult {
    pub position: Position,
    pub exit_price: f64,
    pub exit_time: u64,
    pub reason: ExitReason,
    pub pnl: f64,
    #[allow(dead_code)]
    pub fees: f64,
}

/// Position manager for tracking and closing positions
#[derive(Debug)]
pub struct PositionManager {
    /// Open positions by token_id
    positions: HashMap<String, Position>,
    /// Profit target (spread must narrow by this much)
    profit_target_spread: f64,
    /// Stop loss threshold
    stop_loss_spread: f64,
    /// Maximum hold time in seconds
    max_hold_time: u64,
    /// Closed positions history
    history: Vec<ExitResult>,
}

impl PositionManager {
    pub fn new(profit_target_spread: f64, stop_loss_spread: f64, max_hold_time: u64) -> Self {
        Self {
            positions: HashMap::new(),
            profit_target_spread,
            stop_loss_spread,
            max_hold_time,
            history: Vec::new(),
        }
    }

    /// Add a new position
    pub fn open_position(&mut self, position: Position) {
        println!("📈 [Position] Opened: {} @ ${:.4} (spread: {:.2}%)", 
            position.token_id, position.entry_price, position.entry_spread * 100.0);
        self.positions.insert(position.token_id.clone(), position);
    }

    /// Get all open positions
    pub fn get_positions(&self) -> Vec<&Position> {
        self.positions.values().collect()
    }

    /// Get position by token_id
    #[allow(dead_code)]
    pub fn get_position(&self, token_id: &str) -> Option<&Position> {
        self.positions.get(token_id)
    }

    /// Check positions for exit conditions
    pub fn check_exits(&mut self, markets: &[Market], current_time: u64, fee_rate: f64) -> Vec<ExitResult> {
        let mut exits = Vec::new();
        let mut to_remove = Vec::new();

        for (token_id, position) in &self.positions {
            // Find current market state
            if let Some(market) = markets.iter().find(|m| m.id == position.market_id) {
                let current_spread = market.get_spread();
                let current_price = if position.side == Side::Buy {
                    market.yes_price() // Simplified - should match token
                } else {
                    market.no_price()
                };

                let hold_time = current_time.saturating_sub(position.entry_time);

                // Check exit conditions
                let exit_reason = if current_spread < self.profit_target_spread {
                    // Spread normalized - mean reversion complete
                    Some(ExitReason::MeanReversion)
                } else if current_spread > position.entry_spread + self.stop_loss_spread {
                    // Spread widened - stop loss
                    Some(ExitReason::StopLoss)
                } else if hold_time > self.max_hold_time {
                    // Position timeout
                    Some(ExitReason::Timeout)
                } else {
                    None
                };

                if let Some(reason) = exit_reason {
                    // Calculate PnL
                    let gross_pnl = match position.side {
                        Side::Buy => (current_price - position.entry_price) * position.size,
                        Side::Sell => (position.entry_price - current_price) * position.size,
                    };
                    let fees = position.size * current_price * fee_rate;
                    let net_pnl = gross_pnl - fees;

                    let exit_result = ExitResult {
                        position: position.clone(),
                        exit_price: current_price,
                        exit_time: current_time,
                        reason: reason.clone(),
                        pnl: net_pnl,
                        fees,
                    };

                    println!("📉 [Position] Closed: {} | Reason: {:?} | PnL: ${:.4}", 
                        token_id, reason, net_pnl);

                    exits.push(exit_result);
                    to_remove.push(token_id.clone());
                }
            }
        }

        // Remove closed positions
        for token_id in to_remove {
            if let Some(pos) = self.positions.remove(&token_id) {
                // Already added to exits above
                let _ = pos;
            }
        }

        // Add to history
        self.history.extend(exits.clone());

        exits
    }

    /// Force close a position
    #[allow(dead_code)]
    pub fn close_position(&mut self, token_id: &str, exit_price: f64, fee_rate: f64) -> Option<ExitResult> {
        if let Some(position) = self.positions.remove(token_id) {
            let current_time = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();

            let gross_pnl = match position.side {
                Side::Buy => (exit_price - position.entry_price) * position.size,
                Side::Sell => (position.entry_price - exit_price) * position.size,
            };
            let fees = position.size * exit_price * fee_rate;

            let result = ExitResult {
                position,
                exit_price,
                exit_time: current_time,
                reason: ExitReason::Manual,
                pnl: gross_pnl - fees,
                fees,
            };

            self.history.push(result.clone());
            Some(result)
        } else {
            None
        }
    }

    /// Get total PnL from history
    pub fn total_pnl(&self) -> f64 {
        self.history.iter().map(|e| e.pnl).sum()
    }

    /// Get win rate
    pub fn win_rate(&self) -> f64 {
        if self.history.is_empty() {
            return 0.0;
        }
        let wins = self.history.iter().filter(|e| e.pnl > 0.0).count();
        wins as f64 / self.history.len() as f64
    }

    /// Get trade count
    pub fn trade_count(&self) -> usize {
        self.history.len()
    }
    
    /// Record a simulated trade (for demo mode only)
    pub fn record_simulated_trade(&mut self, pnl: f64) {
        // Create a dummy exit result for stats tracking
        let dummy = ExitResult {
            position: Position {
                market_id: "demo".to_string(),
                token_id: "demo".to_string(),
                side: crate::types::Side::Buy,
                size: 5.0,
                entry_price: 0.5,
                entry_time: 0,
                entry_spread: 0.01,
            },
            exit_price: 0.5,
            exit_time: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            reason: ExitReason::MeanReversion,
            pnl,
            fees: 0.0,
        };
        self.history.push(dummy);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_manager() {
        let mut pm = PositionManager::new(0.01, 0.05, 3600);
        
        let pos = Position {
            market_id: "m1".to_string(),
            token_id: "t1".to_string(),
            side: Side::Buy,
            size: 10.0,
            entry_price: 0.50,
            entry_time: 1000,
            entry_spread: 0.03,
        };
        
        pm.open_position(pos);
        assert_eq!(pm.get_positions().len(), 1);
    }
}
