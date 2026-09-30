use std::collections::HashMap;

#[derive(Clone, Debug, Default)]
pub struct Validator {
    pub id: String,
    pub stake: u64,
    pub commission: u64,
    pub active: bool,
}

impl Validator {
    pub fn new(id: String, stake: u64) -> Self { Self { id, stake, commission: 50, active: true } }
}

#[derive(Clone, Debug, Default)]
pub struct StakePool {
    pub validators: HashMap<String, Validator>,
    pub total_stake: u64,
}

impl StakePool {
    pub fn new() -> Self { Self { validators: HashMap::new(), total_stake: 0 } }

    pub fn add_validator(&mut self, validator: Validator) -> Result<(), String> {
        if validator.id.is_empty() || validator.stake == 0 { return Err("validator id and stake are required".to_string()); }
        if self.validators.contains_key(&validator.id) { return Err("validator already exists".to_string()); }
        self.total_stake = self.total_stake.checked_add(validator.stake).ok_or("total stake overflow")?;
        self.validators.insert(validator.id.clone(), validator);
        Ok(())
    }

    pub fn leader_for_round(&self, round: u64) -> Option<String> {
        let mut entries: Vec<_> = self.validators.values().filter(|v| v.active).collect();
        entries.sort_by(|a,b| a.id.cmp(&b.id));
        if entries.is_empty() { return None; }
        let total: u128 = entries.iter().map(|v| v.stake as u128).sum();
        let mut target = (round as u128) % total;
        for v in entries {
            if target < v.stake as u128 { return Some(v.id.clone()); }
            target -= v.stake as u128;
        }
        None
    }
}
