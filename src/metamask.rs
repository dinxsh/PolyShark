//! MetaMask SDK Integration Module
//!
//! Provides ERC-7715 Advanced Permissions integration for the PolyShark agent.
//! This module handles permission requests, allowance tracking, and transaction submission.

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Permission grant from MetaMask
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionGrant {
    pub permission_id: String,
    pub token: String,
    pub daily_limit: f64,
    pub spent_today: f64,
    pub expires_at: u64,
    pub granted_at: u64,
    pub revoked: bool,
}

/// MetaMask connection status
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
    PermissionPending,
    PermissionGranted,
    PermissionDenied,
}

/// Strategy mode based on remaining allowance
/// Adapts trading behavior to available resources
#[derive(Debug, Clone, PartialEq)]
pub enum StrategyMode {
    /// < 30% allowance remaining - only high-edge trades
    Conservative,
    /// 30-70% allowance - normal trading
    Normal,
    /// > 70% allowance - more frequent trades
    Aggressive,
}

/// Agent operational status
#[derive(Debug, Clone, PartialEq)]
pub enum AgentStatus {
    /// Agent is idle, not trading
    Idle,
    /// Agent is actively trading
    Running,
    /// Agent entered safe mode due to errors
    SafeMode,
    /// Permission has expired
    PermissionExpired,
}

/// MetaMask Smart Account Client
///
/// Handles ERC-7715 permission lifecycle:
/// 1. Request permission from user
/// 2. Track remaining allowance
/// 3. Submit trades via Smart Account
/// 4. Revoke permission when done
#[derive(Debug)]
pub struct MetaMaskClient {
    /// Connection status
    status: Arc<RwLock<ConnectionStatus>>,
    /// Current permission grant (if any)
    permission: Arc<RwLock<Option<PermissionGrant>>>,
    /// User's wallet address
    wallet_address: Arc<RwLock<Option<String>>>,
    /// Snap ID for communication (demo value)
    snap_id: String,
}

impl MetaMaskClient {
    /// Create new MetaMask client
    pub fn new() -> Self {
        Self {
            status: Arc::new(RwLock::new(ConnectionStatus::Disconnected)),
            permission: Arc::new(RwLock::new(None)),
            wallet_address: Arc::new(RwLock::new(None)),
            snap_id: "npm:polyshark-metamask-snap".to_string(),
        }
    }

    /// Get current connection status
    pub async fn get_status(&self) -> ConnectionStatus {
        self.status.read().await.clone()
    }

    /// Check if we have a valid permission
    pub async fn has_valid_permission(&self) -> bool {
        let perm = self.permission.read().await;
        match &*perm {
            Some(p) => !p.revoked && p.expires_at > Self::current_timestamp(),
            None => false,
        }
    }

    /// Get remaining daily allowance
    pub async fn get_remaining_allowance(&self) -> f64 {
        let perm = self.permission.read().await;
        match &*perm {
            Some(p) => (p.daily_limit - p.spent_today).max(0.0),
            None => 0.0,
        }
    }

    /// Get current permission grant
    pub async fn get_permission(&self) -> Option<PermissionGrant> {
        self.permission.read().await.clone()
    }

    /// Get current strategy mode based on remaining allowance
    ///
    /// - Conservative: < 30% remaining (high-edge trades only)
    /// - Normal: 30-70% remaining (standard trading)
    /// - Aggressive: > 70% remaining (more frequent trades)
    pub async fn get_strategy_mode(&self) -> StrategyMode {
        let perm = self.permission.read().await;
        match &*perm {
            Some(p) => {
                let remaining = (p.daily_limit - p.spent_today).max(0.0);
                let percent = remaining / p.daily_limit;

                if percent < 0.30 {
                    StrategyMode::Conservative
                } else if percent > 0.70 {
                    StrategyMode::Aggressive
                } else {
                    StrategyMode::Normal
                }
            }
            None => StrategyMode::Normal,
        }
    }

    /// Get current agent status
    pub async fn get_agent_status(&self) -> AgentStatus {
        let perm = self.permission.read().await;
        match &*perm {
            Some(p) => {
                if p.revoked {
                    AgentStatus::Idle
                } else if p.expires_at < Self::current_timestamp() {
                    AgentStatus::PermissionExpired
                } else {
                    AgentStatus::Running
                }
            }
            None => AgentStatus::Idle,
        }
    }

    /// Set permission from external source (API)
    pub async fn set_permission(&self, grant: PermissionGrant) {
        *self.permission.write().await = Some(grant.clone());
        *self.status.write().await = ConnectionStatus::PermissionGranted;
        println!(
            "✅ [MetaMask] Permission updated via API: {}",
            grant.permission_id
        );
    }

    /// Connect to MetaMask wallet
    ///
    /// In production, this would use window.ethereum or Snap RPC
    /// For demo, we simulate the connection
    pub async fn connect(&self) -> Result<String, MetaMaskError> {
        *self.status.write().await = ConnectionStatus::Connecting;

        // Simulate connection delay
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        // Demo: Generate a fake address
        let address = format!(
            "0x{}",
            hex::encode(&[
                0xDE, 0xAD, 0xBE, 0xEF, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09,
                0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F
            ])
        );

        *self.wallet_address.write().await = Some(address.clone());
        *self.status.write().await = ConnectionStatus::Connected;

        println!(
            "🦊 [MetaMask] Connected to Smart Account: {}",
            &address[..10]
        );

        Ok(address)
    }

    /// Request ERC-7715 spend permission
    ///
    /// This would show a MetaMask popup asking user to approve:
    /// "PolyShark may automatically trade up to {limit} USDC per day"
    pub async fn request_permission(
        &self,
        token: &str,
        daily_limit: f64,
        duration_days: u32,
    ) -> Result<PermissionGrant, MetaMaskError> {
        // Must be connected first
        if *self.status.read().await != ConnectionStatus::Connected {
            return Err(MetaMaskError::NotConnected);
        }

        *self.status.write().await = ConnectionStatus::PermissionPending;

        println!("🔐 [MetaMask] Requesting ERC-7715 Permission...");
        println!("   Token: {}", token);
        println!("   Daily Limit: ${:.2}", daily_limit);
        println!("   Duration: {} days", duration_days);

        // Simulate user approval delay
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

        // Create permission grant
        let now = Self::current_timestamp();
        let grant = PermissionGrant {
            permission_id: format!("perm_{}", now),
            token: token.to_string(),
            daily_limit,
            spent_today: 0.0,
            expires_at: now + (duration_days as u64 * 86400),
            granted_at: now,
            revoked: false,
        };

        *self.permission.write().await = Some(grant.clone());
        *self.status.write().await = ConnectionStatus::PermissionGranted;

        println!("✅ [MetaMask] Permission Granted!");
        println!("   ID: {}", grant.permission_id);
        println!("   Expires: {} days from now", duration_days);

        Ok(grant)
    }

    /// Record a spend against the permission
    pub async fn record_spend(&self, amount: f64) -> Result<(), MetaMaskError> {
        let mut perm = self.permission.write().await;

        match &mut *perm {
            Some(p) => {
                if p.revoked {
                    return Err(MetaMaskError::PermissionRevoked);
                }
                if p.expires_at < Self::current_timestamp() {
                    return Err(MetaMaskError::PermissionExpired);
                }
                if p.spent_today + amount > p.daily_limit {
                    return Err(MetaMaskError::InsufficientAllowance);
                }

                p.spent_today += amount;
                Ok(())
            }
            None => Err(MetaMaskError::NoPermission),
        }
    }

    /// Reset daily spend (called at midnight UTC)
    pub async fn reset_daily_spend(&self) {
        let mut perm = self.permission.write().await;
        if let Some(p) = &mut *perm {
            p.spent_today = 0.0;
            println!("🔄 [MetaMask] Daily allowance reset");
        }
    }

    /// Revoke the current permission
    pub async fn revoke_permission(&self) -> Result<(), MetaMaskError> {
        let mut perm = self.permission.write().await;

        match &mut *perm {
            Some(p) => {
                p.revoked = true;
                *self.status.write().await = ConnectionStatus::Connected;
                println!("🚫 [MetaMask] Permission Revoked: {}", p.permission_id);
                Ok(())
            }
            None => Err(MetaMaskError::NoPermission),
        }
    }

    /// Disconnect from MetaMask
    pub async fn disconnect(&self) {
        *self.permission.write().await = None;
        *self.wallet_address.write().await = None;
        *self.status.write().await = ConnectionStatus::Disconnected;
        println!("👋 [MetaMask] Disconnected");
    }

    fn current_timestamp() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
}

impl Default for MetaMaskClient {
    fn default() -> Self {
        Self::new()
    }
}

/// MetaMask-related errors
#[derive(Debug, Clone)]
pub enum MetaMaskError {
    NotConnected,
    NoPermission,
    PermissionRevoked,
    PermissionExpired,
    PermissionDenied,
    InsufficientAllowance,
    TransactionFailed(String),
    ConnectionFailed(String),
}

impl std::fmt::Display for MetaMaskError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotConnected => write!(f, "MetaMask not connected"),
            Self::NoPermission => write!(f, "No permission granted"),
            Self::PermissionRevoked => write!(f, "Permission has been revoked"),
            Self::PermissionExpired => write!(f, "Permission has expired"),
            Self::PermissionDenied => write!(f, "User denied permission request"),
            Self::InsufficientAllowance => write!(f, "Insufficient daily allowance"),
            Self::TransactionFailed(msg) => write!(f, "Transaction failed: {}", msg),
            Self::ConnectionFailed(msg) => write!(f, "Connection failed: {}", msg),
        }
    }
}

impl std::error::Error for MetaMaskError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_permission_lifecycle() {
        let client = MetaMaskClient::new();

        // Connect
        let addr = client.connect().await.unwrap();
        assert!(addr.starts_with("0x"));
        assert_eq!(client.get_status().await, ConnectionStatus::Connected);

        // Request permission
        let perm = client.request_permission("USDC", 10.0, 30).await.unwrap();
        assert_eq!(perm.daily_limit, 10.0);
        assert!(client.has_valid_permission().await);

        // Check allowance
        assert_eq!(client.get_remaining_allowance().await, 10.0);

        // Record spend
        client.record_spend(3.0).await.unwrap();
        assert_eq!(client.get_remaining_allowance().await, 7.0);

        // Try to overspend
        let result = client.record_spend(8.0).await;
        assert!(matches!(result, Err(MetaMaskError::InsufficientAllowance)));

        // Revoke
        client.revoke_permission().await.unwrap();
        assert!(!client.has_valid_permission().await);
    }
}
