use std::default;

use crate::proposal::Proposal;

#[derive(PartialEq, Eq, Default)]
pub enum Acceptor_Status {
    #[default]
    Proposing,
    Accepted,
    Failed,
    Reject,
    Idle,
}

#[derive(Debug)]
pub struct Promise {
    promised_proposal_id: i32,
    accepted_proposal_id: Option<i32>,
    accepted_value: Option<String>,
}

#[derive(Default)]
pub struct Acceptor {
    status: Acceptor_Status,
    local_generation: usize,
    max_seen_proposal_seq: i32,
    max_seen_accept_seq: i32,
    promised_proposals: Vec<Proposal>,
    accepted_proposals: Vec<Proposal>,
}

pub struct Accept_Message {
    pub status: Acceptor_Status,
    pub proposal_id: i32,
    pub proposal: Proposal,
}

impl Acceptor {
    pub fn new() -> Self {
        Self {
            status: Acceptor_Status::Idle,
            local_generation: 0,
            max_seen_proposal_seq: 0,
            max_seen_accept_seq: 0,
            accepted_proposals: vec![],
            promised_proposals: vec![],
        }
    }

    pub fn prepare(&mut self, proposal: Proposal) -> Option<Promise> {
        if self.max_seen_proposal_seq > proposal.get_proposal_id().clone() {
            return None;
        }
        self.set_max_seen_proposal_seq(proposal.get_proposal_id().clone());
        let last_proposal = self.get_last_accepted_proposal();
        let promised_proposal_id = self.max_seen_proposal_seq as i32;
        let accepted_proposal_id = match last_proposal {
            Some(prop) => Some(prop.get_proposal_id() as i32),
            None => None,
        };
        let accepted_value = match last_proposal {
            Some(prop) => Some(String::from(prop.get_value())),
            None => None,
        };
        let promise = Promise::new(promised_proposal_id, accepted_proposal_id, accepted_value);
        Some(promise)
    }

    fn set_max_seen_proposal_seq(&mut self, value: i32) {
        self.max_seen_proposal_seq = value
    }

    pub fn accept(&mut self, proposal: Proposal) -> Option<Accept_Message> {
        if self.max_seen_proposal_seq > proposal.get_proposal_id() {
            return None;
        }

        let prop = proposal.clone();

        self.max_seen_accept_seq = prop.get_proposal_id().clone();
        self.accepted_proposals.push(prop.clone());

        Some(Accept_Message {
            status: Acceptor_Status::Accepted,
            proposal_id: prop.get_proposal_id().clone(),
            proposal: prop.clone(),
        })
    }

    pub fn get_last_accepted_proposal(&self) -> Option<&Proposal> {
        if self.accepted_proposals.len() > 0 {
            let last_accepted_proposal: &Proposal = self
                .accepted_proposals
                .last()
                .expect("cannot get last proposal");
            Some(last_accepted_proposal)
        } else {
            None
        }
    }
}

impl Promise {
    pub fn new(
        promised_proposal_id: i32,
        accepted_proposal_id: Option<i32>,
        accepted_value: Option<String>,
    ) -> Self {
        Self {
            promised_proposal_id,
            accepted_proposal_id,
            accepted_value,
        }
    }

    pub fn get_promised_proposal_id(&self) -> i32 {
        self.promised_proposal_id
    }
    pub fn get_accepted_proposal_id(&self) -> Option<i32> {
        self.accepted_proposal_id
    }
    pub fn get_accepted_value(&self) -> Option<String> {
        self.accepted_value.clone()
    }
}

impl Accept_Message {
    pub fn get_accepted_proposal(&self) -> Proposal {
        self.proposal.clone()
    }
}
