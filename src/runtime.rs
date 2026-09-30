use crate::{core::{Block, Ledger, Transaction}, storage::RocksDbStore};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

const LEDGER_KEY: &[u8] = b"state/ledger";
fn block_key(height: u64) -> Vec<u8> { format!("blocks/{height:020}").into_bytes() }
fn tx_key(hash: &str) -> Vec<u8> { format!("tx/{hash}").into_bytes() }
fn mempool_key(hash: &str) -> Vec<u8> { format!("mempool/{hash}").into_bytes() }

#[derive(Clone)]
pub struct ChainRuntime {
    pub ledger: Arc<RwLock<Ledger>>,
    pub store: Arc<RocksDbStore>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct RuntimeStatus {
    pub height: u64,
    pub last_hash: String,
    pub tx_count: u64,
}

impl ChainRuntime {
    pub fn open(path: &str) -> Result<Self, String> {
        let store = Arc::new(RocksDbStore::new(path)?);
        let ledger = match store.get_json::<Ledger>(LEDGER_KEY)? {
            Some(ledger) => ledger,
            None => {
                let mut ledger = Ledger::new();
                let genesis: serde_json::Value = serde_json::from_str(include_str!("../genesis/genesis.json"))
                    .map_err(|e| format!("invalid genesis: {e}"))?;
                if let Some(allocations) = genesis.get("allocations").and_then(|v| v.as_array()) {
                    for allocation in allocations {
                        let address = allocation.get("address").and_then(|v| v.as_str()).unwrap_or_default().to_string();
                        let balance = allocation.get("amount").and_then(|v| v.as_str()).and_then(|v| v.parse::<u64>().ok()).unwrap_or(0);
                        if !address.is_empty() {
                            ledger.accounts.insert(address.clone(), crate::core::Account { id: address, balance, nonce: 0 });
                        }
                    }
                }
                store.put_json(LEDGER_KEY, &ledger)?;
                ledger
            }
        };
        Ok(Self { ledger: Arc::new(RwLock::new(ledger)), store })
    }

    pub fn status(&self) -> RuntimeStatus {
        let l = self.ledger.read().expect("ledger lock poisoned");
        RuntimeStatus { height: l.height, last_hash: l.last_hash.clone(), tx_count: l.tx_count }
    }

    pub fn submit_transaction(&self, tx: &Transaction) -> Result<String, String> {
        let ledger = self.ledger.read().map_err(|_| "ledger lock poisoned")?;
        tx.validate(ledger.accounts.get(&tx.sender))?;
        drop(ledger);
        if self.store.get(mempool_key(&tx.hash).as_slice()).is_some() {
            return Err("transaction already in mempool".to_string());
        }
        self.store.put_json(&mempool_key(&tx.hash), tx)?;
        Ok(tx.hash.clone())
    }

    pub fn mempool(&self) -> Result<Vec<Transaction>, String> {
        let mut out = Vec::new();
        for value in self.store.scan_prefix(b"mempool/") {
            out.push(serde_json::from_slice(&value).map_err(|e| e.to_string())?);
        }
        out.sort_by(|a: &Transaction, b: &Transaction| a.hash.cmp(&b.hash));
        Ok(out)
    }

    pub fn build_block(&self, proposer: String, timestamp: u64, max_transactions: usize) -> Result<Block, String> {
        let current = self.ledger.read().map_err(|_| "ledger lock poisoned")?.clone();
        let mut candidate = current.clone();
        let mut transactions = Vec::new();
        for tx in self.mempool()?.into_iter().take(max_transactions) {
            if candidate.apply_transaction(&tx).is_ok() {
                transactions.push(tx);
            }
        }
        let index = current.height.checked_add(1).ok_or("block height overflow")?;
        let mut block = Block {
            index,
            prev_hash: current.last_hash,
            transactions,
            timestamp,
            proposer,
            hash: String::new(),
            gas_used: 0,
        };
        block.hash = block.compute_hash();
        Ok(block)
    }

    pub fn commit_block(&self, block: &Block) -> Result<(), String> {
        let mut ledger = self.ledger.write().map_err(|_| "ledger lock poisoned")?;
        let mut next = ledger.clone();
        next.apply_block(block)?;
        self.store.put_json(&block_key(block.index), block)?;
        for tx in &block.transactions {
            self.store.put_json(&tx_key(&tx.hash), tx)?;
            let _ = self.store.delete(&mempool_key(&tx.hash));
        }
        self.store.put_json(LEDGER_KEY, &next)?;
        *ledger = next;
        Ok(())
    }

    pub fn block(&self, height: u64) -> Result<Option<Block>, String> { self.store.get_json(&block_key(height)) }
    pub fn transaction(&self, hash: &str) -> Result<Option<Transaction>, String> { self.store.get_json(&tx_key(hash)) }

    pub fn account(&self, address: &str) -> Result<Option<crate::core::Account>, String> {
        let ledger = self.ledger.read().map_err(|_| "ledger lock poisoned")?;
        Ok(ledger.accounts.get(address).cloned())
    }
}
