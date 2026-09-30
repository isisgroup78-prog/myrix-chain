use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ValidatorInfo {
    pub id: String,
    pub public_key: String,
    pub stake: u64,
    pub commission: u64,
    pub active: bool,
    pub jailed: bool,
    pub slashed: bool,
}

impl ValidatorInfo {
    pub fn new(id: String, public_key: String, stake: u64) -> Self {
        Self {
            id,
            public_key,
            stake,
            commission: 50,
            active: true,
            jailed: false,
            slashed: false,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ValidatorSet {
    pub validators: HashMap<String, ValidatorInfo>,
    pub total_stake: u64,
    pub epoch: u64,
}

impl ValidatorSet {
    pub fn new() -> Self {
        Self {
            validators: HashMap::new(),
            total_stake: 0,
            epoch: 0,
        }
    }

    pub fn add_validator(&mut self, validator: ValidatorInfo) -> Result<(), String> {
        if validator.stake == 0 {
            return Err("stake must be > 0".to_string());
        }
        if self.validators.contains_key(&validator.id) {
            return Err(format!("validator {} already registered", validator.id));
        }
        self.total_stake += validator.stake;
        self.validators.insert(validator.id.clone(), validator);
        Ok(())
    }

    pub fn get_leader(&self, round: u64) -> Option<String> {
        if self.validators.is_empty() {
            return None;
        }
        let active: Vec<_> = self
            .validators
            .values()
            .filter(|v| v.active && !v.jailed)
            .collect();
        if active.is_empty() {
            return None;
        }
        let idx = (round as usize) % active.len();
        Some(active[idx].id.clone())
    }

    pub fn get_quorum(&self) -> u64 {
        (self.total_stake * 2 / 3) + 1
    }
}
