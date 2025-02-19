use veresiye::Veresiye;

use crate::proposal::{self, Proposal};

#[derive(Default)]
pub struct Learner {
    database: Veresiye,
}

impl Learner {
    pub fn new(database: Veresiye) -> Self {
        Self { database }
    }

    pub fn insert(&mut self, proposal: Proposal) {
        self.database.set(proposal.get_key(), proposal.get_value());
    }

    pub fn read(&mut self, key: String) -> Option<String> {
        self.database.get(&key)
    }
}
