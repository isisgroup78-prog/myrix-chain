use std::collections::HashMap;

#[derive(Clone, Debug, Default)]
pub struct Validator {
    pub id: String,
    pub stake: u64,
    pub commission: u64,
    pub active: bool,
}

impl Validator {
    pub fn new(id: String, stake: u64) -> Self {
        Self {
            id,
            stake,
            commission: 0,
            active: true,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct StakePool {
    pub validators: HashMap<String, Validator>,
    pub total_stake: u64,
}

impl StakePool {
    pub fn new() -> Self {
        Self {
            validators: HashMap::new(),
            total_stake: 0,
        }
    }

    pub fn add_validator(&mut self, validator: Validator) {
        self.total_stake += validator.stake;
        self.validators.insert(validator.id.clone(), validator);
    }

    pub fn leader_for_round(&self, round: u64) -> Option<String> {
        if self.validators.is_empty() {
            return None;
        }

        let mut entries: Vec<_> = self.validators.values().collect();
        entries.sort_by(|a, b| b.stake.cmp(&a.stake));
        let idx = (round as usize) % entries.len();
        Some(entries[idx].id.clone())
    }
}
