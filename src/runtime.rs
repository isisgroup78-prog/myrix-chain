use crate::{
    consensus::{CommitCertificate, Consensus, Vote},
    core::{Block, Ledger, Transaction},
    storage::RocksDbStore,
    validator::ValidatorSet,
};
use ed25519_dalek::SigningKey;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

const LEDGER_KEY: &[u8] = b"state/ledger";
fn block_key(height: u64) -> Vec<u8> { format!("blocks/{height:020}").into_bytes() }
fn tx_key(hash: &str) -> Vec<u8> { format!("tx/{hash}").into_bytes() }
fn mempool_key(hash: &str) -> Vec<u8> { format!("mempool/{hash}").into_bytes() }
fn vote_key(height: u64, round: u64, hash: &str, validator: &str) -> Vec<u8> {
    format!("votes/{height:020}/{round:020}/{hash}/{validator}").into_bytes()
}
fn cert_key(height: u64) -> Vec<u8> { format!("certs/{height:020}").into_bytes() }
fn proposal_key(height: u64) -> Vec<u8> { format!("proposals/{height:020}").into_bytes() }

#[derive(Clone)]
pub struct ChainRuntime {
    pub ledger: Arc<RwLock<Ledger>>,
    pub store: Arc<RocksDbStore>,
    pub validators: Arc<ValidatorSet>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct RuntimeStatus {
    pub height: u64,
    pub last_hash: String,
    pub tx_count: u64,
    pub mempool_size: u64,
    pub validator_count: u64,
    pub finalized_height: u64,
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
        let validators = Arc::new(ValidatorSet::from_genesis()?);
        Ok(Self { ledger: Arc::new(RwLock::new(ledger)), store, validators })
    }

    pub fn status(&self) -> RuntimeStatus {
        let l = self.ledger.read().expect("ledger lock poisoned");
        RuntimeStatus {
            height: l.height,
            last_hash: l.last_hash.clone(),
            tx_count: l.tx_count,
            mempool_size: self.mempool().map(|v| v.len() as u64).unwrap_or(0),
            validator_count: self.validators.active_validators().len() as u64,
            finalized_height: l.height,
        }
    }

    pub fn submit_transaction(&self, tx: &Transaction) -> Result<String, String> {
        let ledger = self.ledger.read().map_err(|_| "ledger lock poisoned")?;
        tx.validate(ledger.accounts.get(&tx.sender))?;
        drop(ledger);
        for existing in self.mempool()? {
            if existing.hash == tx.hash { return Err("transaction already in mempool".to_string()); }
            if existing.sender == tx.sender && existing.nonce == tx.nonce {
                return Err("sender nonce already pending".to_string());
            }
        }
        self.store.put_json(&mempool_key(&tx.hash), tx)?;
        Ok(tx.hash.clone())
    }

    pub fn mempool(&self) -> Result<Vec<Transaction>, String> {
        let mut out = Vec::new();
        for value in self.store.scan_prefix(b"mempool/") {
            out.push(serde_json::from_slice(&value).map_err(|e| e.to_string())?);
        }
        out.sort_by(|a, b| a.sender.cmp(&b.sender).then(a.nonce.cmp(&b.nonce)).then(a.hash.cmp(&b.hash)));
        Ok(out)
    }

    pub fn build_block(
        &self,
        chain_id: &str,
        proposer: String,
        round: u64,
        timestamp: u64,
        max_transactions: usize,
        signing_key: &SigningKey,
    ) -> Result<Block, String> {
        let current = self.ledger.read().map_err(|_| "ledger lock poisoned")?.clone();
        let mut candidate = current.clone();
        let mut transactions = Vec::new();
        for tx in self.mempool()?.into_iter().take(max_transactions) {
            if candidate.apply_transaction(&tx).is_ok() { transactions.push(tx); }
        }
        let index = current.height.checked_add(1).ok_or("block height overflow")?;
        let mut block = Block {
            index,
            round,
            chain_id: chain_id.to_string(),
            prev_hash: current.last_hash,
            transactions,
            timestamp,
            proposer,
            proposer_signature: String::new(),
            hash: String::new(),
            gas_used: 0,
        };
        block.hash = block.compute_hash();
        use ed25519_dalek::Signer;
        block.proposer_signature = hex::encode(signing_key.sign(&block.signing_bytes()).to_bytes());
        Ok(block)
    }

    pub fn store_proposal(&self, block: &Block) -> Result<(), String> {
        self.store.put_json(&proposal_key(block.index), block)
    }

    pub fn proposal(&self, height: u64) -> Result<Option<Block>, String> {
        self.store.get_json(&proposal_key(height))
    }

    pub fn record_vote(&self, vote: &Vote) -> Result<(), String> {
        let prefix = format!("votes/{:020}/{:020}/", vote.height, vote.round).into_bytes();
        for value in self.store.scan_prefix(&prefix) {
            let existing: Vote = serde_json::from_slice(&value).map_err(|e| e.to_string())?;
            if existing.validator_id == vote.validator_id && existing.block_hash != vote.block_hash {
                return Err("validator already voted for another block in this round".to_string());
            }
        }
        let key = vote_key(vote.height, vote.round, &vote.block_hash, &vote.validator_id);
        if self.store.get(&key).is_none() { self.store.put_json(&key, vote)?; }
        Ok(())
    }

    pub fn votes_for(&self, height: u64, round: u64, block_hash: &str) -> Result<Vec<Vote>, String> {
        let prefix = format!("votes/{height:020}/{round:020}/{block_hash}/").into_bytes();
        let mut votes = Vec::new();
        for value in self.store.scan_prefix(&prefix) {
            votes.push(serde_json::from_slice(&value).map_err(|e| e.to_string())?);
        }
        Ok(votes)
    }

    pub fn validate_block(&self, block: &Block, consensus: &Consensus) -> Result<(), String> {
        let ledger = self.ledger.read().map_err(|_| "ledger lock poisoned")?;
        consensus.validate_block(block, &self.validators, ledger.height + 1, &ledger.last_hash)
    }

    pub fn commit_block(&self, block: &Block, cert: &CommitCertificate, consensus: &Consensus) -> Result<(), String> {
        self.validate_block(block, consensus)?;
        consensus.verify_certificate(block, cert, &self.validators)?;
        let mut ledger = self.ledger.write().map_err(|_| "ledger lock poisoned")?;
        if ledger.height + 1 != block.index { return Err("block is not the next height".to_string()); }
        let mut next = ledger.clone();
        next.apply_block(block)?;
        let mut puts = Vec::with_capacity(block.transactions.len() + 3);
        puts.push((block_key(block.index), serde_json::to_vec(block).map_err(|e| e.to_string())?));
        puts.push((LEDGER_KEY.to_vec(), serde_json::to_vec(&next).map_err(|e| e.to_string())?));
        puts.push((cert_key(block.index), serde_json::to_vec(cert).map_err(|e| e.to_string())?));
        let mut deletes = Vec::with_capacity(block.transactions.len());
        for tx in &block.transactions {
            puts.push((tx_key(&tx.hash), serde_json::to_vec(tx).map_err(|e| e.to_string())?));
            deletes.push(mempool_key(&tx.hash));
        }
        self.store.write_batch(puts, deletes)?;
        *ledger = next;
        Ok(())
    }

    pub fn block(&self, height: u64) -> Result<Option<Block>, String> { self.store.get_json(&block_key(height)) }
    pub fn certificate(&self, height: u64) -> Result<Option<CommitCertificate>, String> { self.store.get_json(&cert_key(height)) }
    pub fn transaction(&self, hash: &str) -> Result<Option<Transaction>, String> { self.store.get_json(&tx_key(hash)) }

    pub fn account(&self, address: &str) -> Result<Option<crate::core::Account>, String> {
        let ledger = self.ledger.read().map_err(|_| "ledger lock poisoned")?;
        Ok(ledger.accounts.get(address).cloned())
    }
}
