use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct Account {
    pub id: String,
    pub balance: u64,
    pub nonce: u64,
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct Block {
    pub index: u64,
    pub prev_hash: String,
    pub transactions: Vec<Transaction>,
    pub timestamp: u64,
    pub proposer: String,
    pub hash: String,
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct Transaction {
    pub sender: String,
    pub receiver: String,
    pub amount: u64,
    pub nonce: u64,
    pub signature: String,
    pub hash: String,
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct Ledger {
    pub accounts: std::collections::HashMap<String, Account>,
    pub last_hash: String,
    pub height: u64,
}

impl Ledger {
    pub fn new() -> Self {
        Self {
            accounts: std::collections::HashMap::new(),
            last_hash: "0".to_string(),
            height: 0,
        }
    }
}
