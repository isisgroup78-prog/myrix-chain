use crate::storage::{RocksDbStore, Store};
use revm::primitives::{Address, B256, U256};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::Arc;

const ACCOUNT_PREFIX: &[u8] = b"evm/account/";
const CODE_PREFIX: &[u8] = b"evm/code/";
const STORAGE_PREFIX: &[u8] = b"evm/storage/";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct EvmAccountState {
    pub balance: [u8; 32],
    pub nonce: u64,
    pub code_hash: [u8; 32],
}

impl Default for EvmAccountState {
    fn default() -> Self { Self { balance: [0; 32], nonce: 0, code_hash: [0; 32] } }
}

#[derive(Clone)]
pub struct EvmStateStore {
    store: Arc<RocksDbStore>,
}

impl EvmStateStore {
    pub fn new(store: Arc<RocksDbStore>) -> Self { Self { store } }

    pub fn account(&self, address: Address) -> Result<Option<EvmAccountState>, String> {
        self.store.get_json(&account_key(address))
    }

    pub fn put_account(&self, address: Address, account: &EvmAccountState) -> Result<(), String> {
        self.store.put_json(&account_key(address), account)
    }

    pub fn code(&self, code_hash: B256) -> Option<Vec<u8>> {
        self.store.get(&code_key(code_hash))
    }

    pub fn put_code(&self, code_hash: B256, code: &[u8]) -> Result<(), String> {
        let calculated = Sha256::digest(code);
        if calculated.as_slice() != code_hash.as_slice() {
            return Err("EVM code hash mismatch".to_string());
        }
        self.store.put(&code_key(code_hash), code)
    }

    pub fn storage(&self, address: Address, slot: U256) -> Result<U256, String> {
        Ok(match self.store.get(&storage_key(address, slot)) {
            Some(value) if value.len() == 32 => U256::from_be_slice(&value),
            Some(_) => return Err("invalid persisted EVM storage value".to_string()),
            None => U256::ZERO,
        })
    }

    pub fn put_storage(&self, address: Address, slot: U256, value: U256) -> Result<(), String> {
        self.store.put(&storage_key(address, slot), &value.to_be_bytes::<32>())
    }

    pub fn delete_storage(&self, address: Address, slot: U256) -> Result<(), String> {
        self.store.delete(&storage_key(address, slot))
    }

    /// Deterministic commitment over persisted EVM keys and values.
    /// This is a MYRIX database commitment, not an Ethereum MPT state root.
    pub fn commitment(&self) -> [u8; 32] {
        let mut entries = Vec::new();
        for prefix in [ACCOUNT_PREFIX, CODE_PREFIX, STORAGE_PREFIX] {
            entries.extend(self.store.scan_prefix_entries(prefix));
        }
        entries.sort_by(|a, b| a.0.cmp(&b.0));
        let mut hasher = Sha256::new();
        for (key, value) in entries {
            hasher.update((key.len() as u64).to_be_bytes());
            hasher.update(key);
            hasher.update((value.len() as u64).to_be_bytes());
            hasher.update(value);
        }
        hasher.finalize().into()
    }
}

fn account_key(address: Address) -> Vec<u8> {
    let mut key = ACCOUNT_PREFIX.to_vec();
    key.extend_from_slice(address.as_slice());
    key
}

fn code_key(code_hash: B256) -> Vec<u8> {
    let mut key = CODE_PREFIX.to_vec();
    key.extend_from_slice(code_hash.as_slice());
    key
}

fn storage_key(address: Address, slot: U256) -> Vec<u8> {
    let mut key = STORAGE_PREFIX.to_vec();
    key.extend_from_slice(address.as_slice());
    key.extend_from_slice(&slot.to_be_bytes::<32>());
    key
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_store() -> Arc<RocksDbStore> {
        let suffix = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        Arc::new(RocksDbStore::new(&format!("/tmp/myrix-evm-state-{suffix}")).unwrap())
    }

    #[test]
    fn account_and_storage_round_trip() {
        let state = EvmStateStore::new(temp_store());
        let address = Address::repeat_byte(7);
        let mut account = EvmAccountState::default();
        account.nonce = 3;
        account.balance[31] = 99;
        state.put_account(address, &account).unwrap();
        assert_eq!(state.account(address).unwrap(), Some(account));
        state.put_storage(address, U256::from(4), U256::from(9)).unwrap();
        assert_eq!(state.storage(address, U256::from(4)).unwrap(), U256::from(9));
        assert_ne!(state.commitment(), [0u8; 32]);
    }

    #[test]
    fn code_hash_is_checked() {
        let state = EvmStateStore::new(temp_store());
        let hash = B256::repeat_byte(1);
        assert!(state.put_code(hash, &[1, 2, 3]).is_err());
    }
}
