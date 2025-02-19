use veresiye::Veresiye;

use crate::{
    acceptor::{Accept_Message, Acceptor, Promise},
    learner::Learner,
    proposal::Proposal,
};

#[derive(Default)]
pub struct Proposer {
    acceptor: Acceptor,
    learner: Learner,
}

impl Proposer {
    pub fn new(db_path: String) -> Self {
        let database = Veresiye::new(db_path).unwrap();
        let acceptor = Acceptor::new();
        let learner = Learner::new(database);

        Self { acceptor, learner }
    }

    pub fn prepare(&mut self, proposal: Proposal) -> Option<Promise> {
        match self.acceptor.prepare(proposal) {
            Some(prom) => Some(prom),
            None => None,
        }
    }

    pub fn accept(&mut self, proposal: Proposal) -> Option<Accept_Message> {
        match self.acceptor.accept(proposal) {
            Some(message) => Some(message),
            None => None,
        }
    }

    pub fn commit(&mut self, proposal: Proposal) {
        self.learner.insert(proposal);
    }
}
