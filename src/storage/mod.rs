use rocksdb::DB;
use serde::{de::DeserializeOwned, Serialize};

pub trait Store {
    fn put(&self, key: &[u8], value: &[u8]) -> Result<(), String>;
    fn get(&self, key: &[u8]) -> Option<Vec<u8>>;
    fn delete(&self, key: &[u8]) -> Result<(), String>;
}

pub struct RocksDbStore { db: DB }

impl RocksDbStore {
    pub fn new(path: &str) -> Result<Self, String> { DB::open_default(path).map(|db| Self { db }).map_err(|e| e.to_string()) }

    pub fn put_json<T: Serialize>(&self, key: &[u8], value: &T) -> Result<(), String> {
        let bytes = serde_json::to_vec(value).map_err(|e| e.to_string())?;
        self.put(key, &bytes)
    }

    pub fn get_json<T: DeserializeOwned>(&self, key: &[u8]) -> Result<Option<T>, String> {
        match self.get(key) {
            Some(bytes) => serde_json::from_slice(&bytes).map(Some).map_err(|e| e.to_string()),
            None => Ok(None),
        }
    }
}

impl Store for RocksDbStore {
    fn put(&self, key: &[u8], value: &[u8]) -> Result<(), String> { self.db.put(key, value).map_err(|e| e.to_string()) }
    fn get(&self, key: &[u8]) -> Option<Vec<u8>> { self.db.get(key).ok().flatten() }
    fn delete(&self, key: &[u8]) -> Result<(), String> { self.db.delete(key).map_err(|e| e.to_string()) }
}
