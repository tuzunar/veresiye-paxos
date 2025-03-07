use veresiye::Veresiye;

use crate::{
    acceptor::{Accept_Message, Acceptor, Promise},
    learner::Learner,
    proposal::Proposal,
};

//#[derive(Default)]
pub struct Proposer {
    acceptor: Acceptor,
    learner: Learner,
    last_seen_propose_id: i32,
}

impl Proposer {
    pub fn new(db_path: String) -> Self {
        let database = Veresiye::new(db_path).unwrap();
        let acceptor = Acceptor::new();
        let learner = Learner::new(database);
        let last_seen_propose_id: i32 = 0;

        Self {
            acceptor,
            learner,
            last_seen_propose_id,
        }
    }

    pub fn get_last_seen_propose_id(&self) -> i32 {
        self.last_seen_propose_id
    }

    pub fn set_last_seen_propose_id(&mut self, id: i32) {
        self.last_seen_propose_id = id;
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
