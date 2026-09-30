use prometheus::{Counter, Encoder, Gauge, Registry, TextEncoder};
use std::sync::atomic::{AtomicU64, Ordering};

pub struct MetricsCollector {
    pub registry: Registry,
    pub block_height: Gauge,
    pub tx_count: Counter,
    pub peer_count: Gauge,
    pub validator_count: Gauge,
    last_tx_count: AtomicU64,
}

impl Default for MetricsCollector { fn default() -> Self { Self::new() } }\n\nimpl MetricsCollector {
    pub fn new() -> Self {
        let registry = Registry::new();
        let block_height = Gauge::new("myrix_block_height", "Current block height").unwrap();
        let tx_count = Counter::new("myrix_tx_total", "Total committed transactions").unwrap();
        let peer_count = Gauge::new("myrix_peer_count", "Number of connected peers").unwrap();
        let validator_count = Gauge::new("myrix_validator_count", "Number of active validators").unwrap();
        registry.register(Box::new(block_height.clone())).unwrap();
        registry.register(Box::new(tx_count.clone())).unwrap();
        registry.register(Box::new(peer_count.clone())).unwrap();
        registry.register(Box::new(validator_count.clone())).unwrap();
        Self { registry, block_height, tx_count, peer_count, validator_count, last_tx_count: AtomicU64::new(0) }
    }

    pub fn sync(&self, height: u64, tx_count: u64, validator_count: u64) {
        self.block_height.set(height as f64);
        self.validator_count.set(validator_count as f64);
        let previous = self.last_tx_count.load(Ordering::Relaxed);
        if tx_count > previous {
            self.tx_count.inc_by((tx_count - previous) as f64);
            self.last_tx_count.store(tx_count, Ordering::Relaxed);
        }
    }

    pub fn render(&self) -> Result<String, String> {
        let families = self.registry.gather();
        let mut output = Vec::new();
        TextEncoder::new().encode(&families, &mut output).map_err(|e| e.to_string())?;
        String::from_utf8(output).map_err(|e| e.to_string())
    }
}
