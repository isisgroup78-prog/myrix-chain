pub trait Store {
    fn put(&self, key: &[u8], value: &[u8]) -> Result<(), String>;
    fn get(&self, key: &[u8]) -> Option<Vec<u8>>;
    fn delete(&self, key: &[u8]) -> Result<(), String>;
}

use rocksdb::DB;

pub struct RocksDbStore {
    db: DB,
}

impl RocksDbStore {
    pub fn new(path: &str) -> Result<Self, String> {
        let db = DB::open_default(path).map_err(|e| e.to_string())?;
        Ok(Self { db })
    }
}

impl Store for RocksDbStore {
    fn put(&self, key: &[u8], value: &[u8]) -> Result<(), String> {
        self.db
            .put(key, value)
            .map_err(|e| e.to_string())
            .map(|_| ())
    }

    fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        self.db.get(key).ok().flatten()
    }

    fn delete(&self, key: &[u8]) -> Result<(), String> {
        self.db
            .delete(key)
            .map_err(|e| e.to_string())
            .map(|_| ())
    }
}
