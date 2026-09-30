use crate::{core::{Block, Ledger}, security, validator::ValidatorSet};
use ed25519_dalek::SigningKey;
use serde::{Deserialize, Serialize};

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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Vote {
    pub validator_id: String,
    pub chain_id: String,
    pub height: u64,
    pub round: u64,
    pub block_hash: String,
    pub signature: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CommitCertificate {
    pub chain_id: String,
    pub height: u64,
    pub round: u64,
    pub block_hash: String,
    pub voters: Vec<String>,
}

impl Consensus {
    pub fn new() -> Self {
        Self { epoch: 0, config: ConsensusConfig { epoch_length: 100, quorum_bps: 6700, min_validators: 3 } }
    }

    pub fn advance_epoch(&mut self) { self.epoch = self.epoch.saturating_add(1); }

    pub fn leader_for_round(&self, round: u64, validators: &ValidatorSet) -> Option<String> {
        validators.get_leader(round + self.epoch.saturating_mul(self.config.epoch_length))
    }

    pub fn vote_signing_bytes(chain_id: &str, height: u64, round: u64, block_hash: &str) -> Vec<u8> {
        format!("MYRIX-VOTE-V1|{chain_id}|{height}|{round}|{block_hash}").into_bytes()
    }

    pub fn validate_block(&self, block: &Block, validators: &ValidatorSet, ledger: &Ledger, expected_chain_id: &str) -> Result<(), String> {
        if validators.active_validators().len() < self.config.min_validators as usize {
            return Err("not enough active validators".to_string());
        }
        if !validators.contains_active(&block.proposer) { return Err("proposer is not an active validator".to_string()); }
        block.validate_header(ledger.height + 1, &ledger.last_hash, expected_chain_id)?;
        let expected = self.leader_for_round(block.round, validators).ok_or("no leader available")?;
        if expected != block.proposer { return Err("block proposer is not the deterministic leader".to_string()); }
        let validator = validators.validators.get(&block.proposer).ok_or("unknown proposer")?;
        security::verify_ed25519(&validator.public_key, &block.signing_bytes(), &block.proposer_signature)?;
        let mut candidate = ledger.clone();
        for tx in &block.transactions { candidate.apply_transaction(tx)?; }
        Ok(())
    }

    pub fn sign_vote(&self, validator_id: &str, chain_id: &str, height: u64, round: u64, block_hash: &str, key: &SigningKey) -> Vote {
        let bytes = Self::vote_signing_bytes(chain_id, height, round, block_hash);
        use ed25519_dalek::Signer;
        Vote {
            validator_id: validator_id.to_string(),
            chain_id: chain_id.to_string(),
            height,
            round,
            block_hash: block_hash.to_string(),
            signature: hex::encode(key.sign(&bytes).to_bytes()),
        }
    }

    pub fn verify_vote(&self, vote: &Vote, validators: &ValidatorSet) -> Result<(), String> {
        let validator = validators.validators.get(&vote.validator_id).ok_or("unknown validator")?;
        if !validator.active || validator.jailed || validator.slashed { return Err("validator is not eligible to vote".to_string()); }
        security::verify_ed25519(
            &validator.public_key,
            &Self::vote_signing_bytes(&vote.chain_id, vote.height, vote.round, &vote.block_hash),
            &vote.signature,
        )
    }

    pub fn build_certificate(&self, block: &Block, votes: &[Vote], validators: &ValidatorSet) -> Result<CommitCertificate, String> {
        let mut voters = Vec::new();
        for vote in votes {
            if vote.chain_id == block.chain_id && vote.height == block.index && vote.round == block.round
                && vote.block_hash == block.hash && self.verify_vote(vote, validators).is_ok()
                && !voters.contains(&vote.validator_id) {
                voters.push(vote.validator_id.clone());
            }
        }
        if !self.has_quorum(validators, &voters) { return Err("quorum not reached".to_string()); }
        Ok(CommitCertificate {
            chain_id: block.chain_id.clone(),
            height: block.index,
            round: block.round,
            block_hash: block.hash.clone(),
            voters,
        })
    }

    pub fn has_quorum(&self, validators: &ValidatorSet, voters: &[String]) -> bool {
        validators.voting_power(voters) >= validators.quorum()
    }

    pub fn verify_certificate(&self, block: &Block, cert: &CommitCertificate, validators: &ValidatorSet) -> Result<(), String> {
        if cert.chain_id != block.chain_id || cert.height != block.index || cert.round != block.round || cert.block_hash != block.hash {
            return Err("certificate does not match block".to_string());
        }
        let mut unique = Vec::new();
        for id in &cert.voters {
            if unique.contains(id) { return Err("duplicate voter in certificate".to_string()); }
            let v = validators.validators.get(id).ok_or("certificate contains unknown validator")?;
            if !v.active || v.jailed || v.slashed { return Err("certificate contains ineligible validator".to_string()); }
            unique.push(id.clone());
        }
        if !self.has_quorum(validators, &unique) { return Err("certificate quorum not reached".to_string()); }
        Ok(())
    }
}
