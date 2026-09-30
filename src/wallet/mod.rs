use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct Wallet {
    pub address: String,
    pub public_key: String,
    pub private_key: String,
}

impl Wallet {
    pub fn new() -> Self {
        Self {
            address: "0xwallet_001".to_string(),
            public_key: "pub-key-001".to_string(),
            private_key: "priv-key-001".to_string(),
        }
    }
}
