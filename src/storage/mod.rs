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

    pub fn write_batch(&self, puts: Vec<(Vec<u8>, Vec<u8>)>, deletes: Vec<Vec<u8>>) -> Result<(), String> {
        let mut batch = rocksdb::WriteBatch::default();
        for (key, value) in puts { batch.put(key, value); }
        for key in deletes { batch.delete(key); }
        self.db.write(batch).map_err(|e| e.to_string())
    }

    pub fn scan_prefix(&self, prefix: &[u8]) -> Vec<Vec<u8>> {
        use rocksdb::{Direction, IteratorMode};
        self.db.iterator(IteratorMode::From(prefix, Direction::Forward))
            .take_while(|item| item.as_ref().map(|(k, _)| k.starts_with(prefix)).unwrap_or(false))
            .filter_map(|item| item.ok().map(|(_, v)| v.to_vec()))
            .collect()
    }
}

impl Store for RocksDbStore {
    fn put(&self, key: &[u8], value: &[u8]) -> Result<(), String> { self.db.put(key, value).map_err(|e| e.to_string()) }
    fn get(&self, key: &[u8]) -> Option<Vec<u8>> { self.db.get(key).ok().flatten() }
    fn delete(&self, key: &[u8]) -> Result<(), String> { self.db.delete(key).map_err(|e| e.to_string()) }
}
