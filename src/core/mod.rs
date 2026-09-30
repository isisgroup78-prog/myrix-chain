use ed25519_dalek::{Signature, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

fn sha256_hex(data: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(data);
    hex::encode(h.finalize())
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct Account {
    pub id: String,
    pub balance: u64,
    pub nonce: u64,
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct Transaction {
    pub sender: String,
    pub receiver: String,
    pub amount: u64,
    pub nonce: u64,
    pub fee: u64,
    pub public_key: String,
    pub signature: String,
    pub hash: String,
}

impl Transaction {
    pub fn signing_bytes(&self) -> Vec<u8> {
        format!("{}|{}|{}|{}|{}|{}", self.sender, self.receiver, self.amount, self.nonce, self.fee, self.public_key).into_bytes()
    }

    pub fn compute_hash(&self) -> String {
        sha256_hex(&self.signing_bytes())
    }

    pub fn verify_signature(&self) -> Result<(), String> {
        let pk = hex::decode(self.public_key.strip_prefix("ed25519:").unwrap_or(&self.public_key))
            .map_err(|_| "invalid public key encoding".to_string())?;
        let sig = hex::decode(&self.signature).map_err(|_| "invalid signature encoding".to_string())?;
        let pk: [u8; 32] = pk.try_into().map_err(|_| "public key must be 32 bytes".to_string())?;
        let sig: [u8; 64] = sig.try_into().map_err(|_| "signature must be 64 bytes".to_string())?;
        let key = VerifyingKey::from_bytes(&pk).map_err(|_| "invalid ed25519 public key".to_string())?;
        let signature = Signature::from_bytes(&sig);
        ed25519_dalek::Verifier::verify(&key, &self.signing_bytes(), &signature)
            .map_err(|_| "invalid transaction signature".to_string())
    }

    pub fn validate(&self, account: Option<&Account>) -> Result<(), String> {
        if self.sender.is_empty() || self.receiver.is_empty() {
            return Err("sender and receiver are required".to_string());
        }
        if self.amount == 0 {
            return Err("amount must be > 0".to_string());
        }
        if self.compute_hash() != self.hash {
            return Err("transaction hash mismatch".to_string());
        }
        self.verify_signature()?;
        if let Some(account) = account {
            if account.nonce != self.nonce {
                return Err(format!("invalid nonce: expected {}", account.nonce));
            }
            let total = self.amount.checked_add(self.fee).ok_or("amount overflow")?;
            if account.balance < total {
                return Err("insufficient balance".to_string());
            }
        }
        Ok(())
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct Block {
    pub index: u64,
    pub prev_hash: String,
    pub transactions: Vec<Transaction>,
    pub timestamp: u64,
    pub proposer: String,
    pub hash: String,
    pub gas_used: u64,
}

impl Block {
    pub fn compute_hash(&self) -> String {
        let tx_hashes = self.transactions.iter().map(|t| t.hash.as_str()).collect::<Vec<_>>().join(",");
        sha256_hex(format!("{}|{}|{}|{}|{}|{}", self.index, self.prev_hash, tx_hashes, self.timestamp, self.proposer, self.gas_used).as_bytes())
    }

    pub fn validate_header(&self, expected_height: u64, expected_prev_hash: &str) -> Result<(), String> {
        if self.index != expected_height {
            return Err("invalid block height".to_string());
        }
        if self.prev_hash != expected_prev_hash {
            return Err("invalid previous hash".to_string());
        }
        if self.hash != self.compute_hash() {
            return Err("block hash mismatch".to_string());
        }
        if self.proposer.is_empty() {
            return Err("missing proposer".to_string());
        }
        Ok(())
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct Ledger {
    pub accounts: HashMap<String, Account>,
    pub last_hash: String,
    pub height: u64,
    pub tx_count: u64,
}

impl Ledger {
    pub fn new() -> Self {
        Self { accounts: HashMap::new(), last_hash: "0".to_string(), height: 0, tx_count: 0 }
    }

    pub fn apply_transaction(&mut self, tx: &Transaction) -> Result<(), String> {
        let sender = self.accounts.get(&tx.sender).cloned();
        tx.validate(sender.as_ref())?;
        let total = tx.amount.checked_add(tx.fee).ok_or("amount overflow")?;
        let sender = self.accounts.entry(tx.sender.clone()).or_insert(Account { id: tx.sender.clone(), balance: 0, nonce: 0 });
        sender.balance -= total;
        sender.nonce = sender.nonce.checked_add(1).ok_or("nonce overflow")?;
        let receiver = self.accounts.entry(tx.receiver.clone()).or_insert(Account { id: tx.receiver.clone(), balance: 0, nonce: 0 });
        receiver.balance = receiver.balance.checked_add(tx.amount).ok_or("balance overflow")?;
        self.tx_count = self.tx_count.checked_add(1).ok_or("transaction count overflow")?;
        Ok(())
    }

    pub fn apply_block(&mut self, block: &Block) -> Result<(), String> {
        block.validate_header(self.height + 1, &self.last_hash)?;
        for tx in &block.transactions { self.apply_transaction(tx)?; }
        self.height = block.index;
        self.last_hash = block.hash.clone();
        Ok(())
    }
}
