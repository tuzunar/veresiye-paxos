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

pub struct Promise {
    promised_proposal_id: usize,
    accepted_proposal_id: Option<usize>,
    accepted_value: Option<String>,
}

#[derive(Default)]
pub struct Acceptor {
    status: Acceptor_Status,
    local_generation: usize,
    max_seen_proposal_seq: usize,
    max_seen_accept_seq: usize,
    promised_proposals: Vec<Proposal>,
    accepted_proposals: Vec<Proposal>,
}

pub struct Accept_Message {
    pub status: Acceptor_Status,
    pub proposal_id: usize,
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
        if self.max_seen_proposal_seq > proposal.get_seq_id().clone() {
            return None;
        }
        self.max_seen_proposal_seq = usize::from(proposal.get_seq_id().clone());
        let last_proposal = self.get_last_accepted_proposal();
        Some(Promise {
            promised_proposal_id: self.max_seen_proposal_seq,
            accepted_proposal_id: match last_proposal {
                Some(prop) => Some(prop.get_seq_id()),
                None => None,
            },
            accepted_value: match last_proposal {
                Some(prop) => Some(String::from(prop.get_value())),
                None => None,
            },
        })
    }

    pub fn accept(&mut self, proposal: Proposal) -> Option<Accept_Message> {
        if self.max_seen_proposal_seq > proposal.get_seq_id() {
            return None;
        }

        let prop = proposal.clone();

        self.max_seen_accept_seq = prop.get_seq_id().clone();
        self.accepted_proposals.push(prop.clone());

        Some(Accept_Message {
            status: Acceptor_Status::Accepted,
            proposal_id: prop.get_seq_id().clone(),
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

impl Accept_Message {
    pub fn get_accepted_proposal(&self) -> Proposal {
        self.proposal.clone()
    }
}
