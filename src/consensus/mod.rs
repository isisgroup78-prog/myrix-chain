#[derive(Clone, Debug)]
pub struct ConsensusConfig {
    pub epoch_length: u64,
    pub quorum: u64,
    pub min_validators: u64,
}

#[derive(Clone, Debug)]
pub struct Consensus {
    pub epoch: u64,
    pub leader: String,
    pub config: ConsensusConfig,
}

impl Consensus {
    pub fn new() -> Self {
        Self {
            epoch: 0,
            leader: "validator-1".to_string(),
            config: ConsensusConfig {
                epoch_length: 100,
                quorum: 67,
                min_validators: 3,
            },
        }
    }

    pub fn advance_epoch(&mut self) {
        self.epoch += 1;
    }

    pub fn validate_block(&self, proposer: &str, active_validators: usize) -> bool {
        !proposer.is_empty() && active_validators >= self.config.min_validators as usize
    }
}
