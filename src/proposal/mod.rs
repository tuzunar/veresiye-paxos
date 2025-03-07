#[derive(Debug, Clone, Default)]
pub struct Proposal {
    proposer_id: i32,
    key: String,
    value: String,
}

impl Proposal {
    pub fn new(proposer_id: i32, key: String, value: String) -> Self {
        Self {
            proposer_id,
            key,
            value,
        }
    }

    pub fn get_proposal_id(&self) -> i32 {
        self.proposer_id
    }

    pub fn get_key(&self) -> &str {
        &self.key
    }

    pub fn get_value(&self) -> &str {
        &self.value
    }
}
