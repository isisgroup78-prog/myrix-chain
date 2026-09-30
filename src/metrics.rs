use prometheus::{Encoder, Gauge, Counter, Registry, TextEncoder};

pub struct MetricsCollector {
    pub registry: Registry,
    pub block_height: Gauge,
    pub tx_count: Counter,
    pub peer_count: Gauge,
    pub validator_count: Gauge,
}

impl MetricsCollector {
    pub fn new() -> Self {
        let registry = Registry::new();
        let block_height = Gauge::new("myrix_block_height", "Current block height").unwrap();
        let tx_count = Counter::new("myrix_tx_total", "Total transactions").unwrap();
        let peer_count = Gauge::new("myrix_peer_count", "Number of peers").unwrap();
        let validator_count = Gauge::new("myrix_validator_count", "Number of validators").unwrap();
        registry.register(Box::new(block_height.clone())).unwrap();
        registry.register(Box::new(tx_count.clone())).unwrap();
        registry.register(Box::new(peer_count.clone())).unwrap();
        registry.register(Box::new(validator_count.clone())).unwrap();
        Self { registry, block_height, tx_count, peer_count, validator_count }
    }

    pub fn render(&self) -> Result<String, String> {
        let families = self.registry.gather();
        let mut output = Vec::new();
        TextEncoder::new().encode(&families, &mut output).map_err(|e| e.to_string())?;
        String::from_utf8(output).map_err(|e| e.to_string())
    }
}
