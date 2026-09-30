use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub network_name: String,
    pub chain_id: String,
    pub api_addr: String,
    pub validator_key: Option<String>,
    pub node_id: String,
    pub max_peers: u32,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            network_name: std::env::var("NETWORK_NAME").unwrap_or_else(|_| "MYRIX Chain".to_string()),
            chain_id: std::env::var("CHAIN_ID").unwrap_or_else(|_| "myrix-mainnet-1".to_string()),
            api_addr: std::env::var("API_ADDR").unwrap_or_else(|_| "0.0.0.0:3000".to_string()),
            validator_key: std::env::var("VALIDATOR_KEY").ok(),
            node_id: std::env::var("NODE_ID").unwrap_or_else(|_| uuid::Uuid::new_v4().to_string()),
            max_peers: std::env::var("MAX_PEERS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(128),
        }
    }
}
