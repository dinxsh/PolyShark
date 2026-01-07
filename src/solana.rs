use solana_client::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use std::error::Error;

pub struct SolanaManager {
    client: RpcClient,
}

impl SolanaManager {
    /// Connect to Solana Devnet
    pub fn new() -> Self {
        // Use standard Devnet URL
        let url = "https://api.devnet.solana.com".to_string();
        // Commitment: confirmed is usually good balance of speed/safety for bots
        let client = RpcClient::new_with_commitment(url, CommitmentConfig::confirmed());

        Self { client }
    }

    /// Verify connection by fetching cluster version
    pub fn check_connection(&self) -> Result<String, Box<dyn Error>> {
        let version = self.client.get_version()?;
        Ok(version.solana_core)
    }

    /// (Mock) Get demo wallet balance or real if pubkey provided
    /// For this hackathon, we just show we *can* talk to the chain.
    #[allow(dead_code)]
    pub fn get_demo_balance(&self) -> Result<f64, Box<dyn Error>> {
        // Just checking a known active devnet account or random would be flaky if empty.
        // For the demo "Readiness", getting the version is the proof of connectivity.
        // But let's implement a dummy balance fetch for a known faucet or similar if we wanted.
        // For now, let's just return a placeholder or 0.0 if not funded.
        Ok(0.0)
    }
}
