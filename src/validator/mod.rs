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
        Self { id, public_key, stake, commission: 50, active: true, jailed: false, slashed: false }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ValidatorSet {
    pub validators: HashMap<String, ValidatorInfo>,
    pub total_stake: u64,
    pub epoch: u64,
}

impl ValidatorSet {
    pub fn new() -> Self { Self { validators: HashMap::new(), total_stake: 0, epoch: 0 } }

    pub fn add_validator(&mut self, validator: ValidatorInfo) -> Result<(), String> {
        if validator.id.is_empty() || validator.stake == 0 { return Err("validator id and stake are required".to_string()); }
        if self.validators.contains_key(&validator.id) { return Err(format!("validator {} already registered", validator.id)); }
        self.total_stake = self.total_stake.checked_add(validator.stake).ok_or("total stake overflow")?;
        self.validators.insert(validator.id.clone(), validator);
        Ok(())
    }

    pub fn active_validators(&self) -> Vec<&ValidatorInfo> {
        let mut active: Vec<_> = self.validators.values().filter(|v| v.active && !v.jailed && !v.slashed).collect();
        active.sort_by(|a,b| a.id.cmp(&b.id));
        active
    }

    pub fn contains_active(&self, id: &str) -> bool {
        self.validators.get(id).map(|v| v.active && !v.jailed && !v.slashed).unwrap_or(false)
    }

    pub fn get_leader(&self, round: u64) -> Option<String> {
        let active = self.active_validators();
        if active.is_empty() { return None; }
        let total: u128 = active.iter().map(|v| v.stake as u128).sum();
        if total == 0 { return None; }
        let mut target = (round as u128) % total;
        for v in active {
            let stake = v.stake as u128;
            if target < stake { return Some(v.id.clone()); }
            target -= stake;
        }
        None
    }

    pub fn active_stake(&self) -> u64 {
        self.validators.values()
            .filter(|v| v.active && !v.jailed && !v.slashed)
            .map(|v| v.stake)
            .sum()
    }

    pub fn quorum(&self) -> u64 {
        let active = self.active_stake();
        (active.saturating_mul(2) / 3).saturating_add(1)
    }

    pub fn voting_power(&self, ids: &[String]) -> u64 {
        ids.iter().filter_map(|id| self.validators.get(id)).filter(|v| v.active && !v.jailed && !v.slashed).map(|v| v.stake).sum()
    }
}
