#[derive(Clone, Debug, Default)]
pub struct RollupBatch {
    pub batch_id: String,
    pub tx_count: u64,
}

#[derive(Clone, Debug, Default)]
pub struct OptimisticRollup {
    pub pending_batches: Vec<RollupBatch>,
}

impl OptimisticRollup {
    pub fn new() -> Self {
        Self {
            pending_batches: Vec::new(),
        }
    }

    pub fn submit_batch(&mut self, batch_id: String, tx_count: u64) {
        self.pending_batches.push(RollupBatch { batch_id, tx_count });
    }
}
