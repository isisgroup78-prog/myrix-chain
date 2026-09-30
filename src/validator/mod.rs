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

#[derive(Deserialize)]
struct GenesisValidator {
    id: String,
    public_key: String,
    stake: String,
    #[serde(default)]
    commission_bps: u64,
}

impl ValidatorSet {
    pub fn from_genesis() -> Result<Self, String> {
        let raw = include_str!("../../genesis/genesis.json");
        let value: serde_json::Value = serde_json::from_str(raw).map_err(|e| format!("invalid genesis: {e}"))?;
        let items = value.get("validators").and_then(|v| v.as_array()).ok_or("genesis validators missing")?;
        let mut set = Self::new();
        for item in items {
            let v: GenesisValidator = serde_json::from_value(item.clone()).map_err(|e| format!("invalid genesis validator: {e}"))?;
            let stake = v.stake.parse::<u64>().map_err(|_| format!("invalid stake for validator {}", v.id))?;
            let mut info = ValidatorInfo::new(v.id, v.public_key, stake);
            info.commission = v.commission_bps;
            set.add_validator(info)?;
        }
        Ok(set)
    }


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
            .try_fold(0u64, |total, v| total.checked_add(v.stake))
            .unwrap_or(u64::MAX)
    }

    pub fn quorum(&self) -> u64 {
        let active = self.active_stake() as u128;
        ((active * 2) / 3 + 1).min(u64::MAX as u128) as u64
    }

    pub fn voting_power(&self, ids: &[String]) -> u64 {
        ids.iter()
            .filter_map(|id| self.validators.get(id))
            .filter(|v| v.active && !v.jailed && !v.slashed)
            .fold(0u64, |total, v| total.saturating_add(v.stake))
    }
}
