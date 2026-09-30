use crate::{core::{Block, Ledger, Transaction}, storage::RocksDbStore};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

const LEDGER_KEY: &[u8] = b"state/ledger";
fn block_key(height: u64) -> Vec<u8> { format!("blocks/{height:020}").into_bytes() }
fn tx_key(hash: &str) -> Vec<u8> { format!("tx/{hash}").into_bytes() }

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
        let ledger = store.get_json::<Ledger>(LEDGER_KEY)?.unwrap_or_else(Ledger::new);
        Ok(Self { ledger: Arc::new(RwLock::new(ledger)), store })
    }

    pub fn status(&self) -> RuntimeStatus {
        let l = self.ledger.read().expect("ledger lock poisoned");
        RuntimeStatus { height: l.height, last_hash: l.last_hash.clone(), tx_count: l.tx_count }
    }

    pub fn submit_transaction(&self, tx: &Transaction) -> Result<String, String> {
        let hash = tx.hash.clone();
        let mut ledger = self.ledger.write().map_err(|_| "ledger lock poisoned")?;
        let mut next = ledger.clone();
        next.apply_transaction(tx)?;
        self.store.put_json(&format!("mempool/{hash}").into_bytes(), tx)?;
        *ledger = next;
        self.store.put_json(LEDGER_KEY, &*ledger)?;
        Ok(hash)
    }

    pub fn commit_block(&self, block: &Block) -> Result<(), String> {
        let mut ledger = self.ledger.write().map_err(|_| "ledger lock poisoned")?;
        let mut next = ledger.clone();
        next.apply_block(block)?;
        self.store.put_json(&block_key(block.index), block)?;
        for tx in &block.transactions {
            self.store.put_json(&tx_key(&tx.hash), tx)?;
            let _ = self.store.delete(format!("mempool/{}", tx.hash).as_bytes());
        }
        self.store.put_json(LEDGER_KEY, &next)?;
        *ledger = next;
        Ok(())
    }

    pub fn block(&self, height: u64) -> Result<Option<Block>, String> {
        self.store.get_json(&block_key(height))
    }

    pub fn transaction(&self, hash: &str) -> Result<Option<Transaction>, String> {
        self.store.get_json(&tx_key(hash))
    }

    pub fn account(&self, address: &str) -> Result<Option<crate::core::Account>, String> {
        let ledger = self.ledger.read().map_err(|_| "ledger lock poisoned")?;
        Ok(ledger.accounts.get(address).cloned())
    }
}
