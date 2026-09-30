use std::collections::HashMap;

#[derive(Clone, Debug, Default)]
pub struct ProposalVote {
    pub voter: String,
    pub weight: u64,
    pub choice: String,
}

#[derive(Clone, Debug, Default)]
pub struct Governance {
    pub proposals: HashMap<String, Vec<ProposalVote>>,
}

impl Governance {
    pub fn new() -> Self {
        Self {
            proposals: HashMap::new(),
        }
    }

    pub fn add_vote(&mut self, proposal_id: String, voter: String, weight: u64, choice: String) {
        self.proposals
            .entry(proposal_id)
            .or_default()
            .push(ProposalVote {
                voter,
                weight,
                choice,
            });
    }
}
