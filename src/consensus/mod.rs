use crate::core::Block;
use crate::validator::ValidatorSet;

#[derive(Clone, Debug)]
pub struct ConsensusConfig {
    pub epoch_length: u64,
    pub quorum_bps: u64,
    pub min_validators: u64,
}

#[derive(Clone, Debug)]
pub struct Consensus {
    pub epoch: u64,
    pub config: ConsensusConfig,
}

impl Consensus {
    pub fn new() -> Self {
        Self { epoch: 0, config: ConsensusConfig { epoch_length: 100, quorum_bps: 6700, min_validators: 3 } }
    }

    pub fn advance_epoch(&mut self) { self.epoch = self.epoch.saturating_add(1); }

    pub fn leader_for_round(&self, round: u64, validators: &ValidatorSet) -> Option<String> {
        validators.get_leader(round + self.epoch.saturating_mul(self.config.epoch_length))
    }

    pub fn validate_block(&self, block: &Block, validators: &ValidatorSet, expected_height: u64, prev_hash: &str) -> Result<(), String> {
        if validators.active_validators().len() < self.config.min_validators as usize {
            return Err("not enough active validators".to_string());
        }
        if !validators.contains_active(&block.proposer) {
            return Err("proposer is not an active validator".to_string());
        }
        block.validate_header(expected_height, prev_hash)?;
        let expected = self.leader_for_round(block.index, validators).ok_or("no leader available")?;
        if expected != block.proposer {
            return Err("block proposer is not the deterministic leader".to_string());
        }
        for tx in &block.transactions {
            tx.validate(None)?;
        }
        Ok(())
    }

    pub fn has_quorum(&self, validators: &ValidatorSet, voters: &[String]) -> bool {
        validators.voting_power(voters) >= validators.quorum()
    }
}
