use serde::Deserialize;
use solana_sdk::{pubkey::Pubkey, signature::Signature};

#[derive(Deserialize, Debug, Clone)]
pub struct Target {
    pub address: Pubkey,
    pub before: Option<Signature>,
    pub until: Option<Signature>,
}

pub const DEFAULT_RPC_ENDPOINT: &str = "https://api.mainnet-beta.solana.com";
pub const DEFAULT_SIGNATURE_FETCH_LIMIT: usize = 1_000;
pub const DEFAULT_MAX_PENDING_SIGNATURES: usize = 10_000;
