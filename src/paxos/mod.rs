use std::net::SocketAddr;
use std::sync::Arc;

use ::tokio::sync::Mutex;
use tonic::transport::Channel;

use crate::proto::{InsertRequest, InsertResponse, RegisterRequest, RegisterResponse};
use crate::{acceptor::Acceptor, learner::Learner, proposal::Proposal, proposer::Proposer};
use crate::{
    proposer,
    proto::{
        paxos_server::{Paxos, PaxosServer},
        AcceptorRequest, AcceptorResponse, LearnerRequest, LearnerResponse, PingRequest,
        PingResponse, ProposerRequest, ProposerResponse,
    },
};
use tonic::{Request, Response, Status};

pub struct PaxosService {
    node_id: i32,
    addr: SocketAddr,
    clusters: Arc<Mutex<Vec<NodeConfig>>>,
    connections: Arc<Mutex<Vec<Channel>>>,
    proposer: Arc<Mutex<Proposer>>,
    leader_id: Arc<Mutex<Option<i32>>>,
}

pub struct NodeConfig {
    node_id: i32,
    addr: SocketAddr,
    status: Connection_Status,
}

enum Connection_Status {
    Active,
    Unreachable,
    Retrying,
}

impl PaxosService {
    pub fn new(
        node_id: i32,
        addr: SocketAddr,
        clusters: Arc<Mutex<Vec<NodeConfig>>>,
        connections: Arc<Mutex<Vec<Channel>>>,
        proposer: Arc<Mutex<Proposer>>,
        leader_id: Option<i32>,
    ) -> Self {
        let mut leader: i32 = 0;
        let id: i32 = match leader_id {
            Some(id) => id,
            None => node_id,
        };

        Self {
            node_id,
            addr,
            clusters,
            connections,
            proposer,
            leader_id: Arc::new(Mutex::new(Some(id))),
        }
    }
}

#[tonic::async_trait]
impl Paxos for PaxosService {
    async fn ping(&self, request: Request<PingRequest>) -> Result<Response<PingResponse>, Status> {
        let reply = PingResponse { health: true };

        Ok(Response::new(reply))
    }

    async fn register(
        &self,
        request: Request<RegisterRequest>,
    ) -> Result<Response<RegisterResponse>, Status> {
        let req = request.get_ref();

        let mut clusters = self.clusters.lock().await;

        let address: SocketAddr = req.addr.clone().parse().unwrap();

        let node = NodeConfig::new(req.node_id.clone(), address, Connection_Status::Active);

        clusters.push(node);

        let reply = RegisterResponse {
            register_status: true,
        };

        Ok(Response::new(reply))
    }

    async fn send_register(
        &self,
        request: Request<RegisterRequest>,
    ) -> Result<Response<RegisterResponse>, Status> {
    }

    async fn insert(
        &self,
        request: Request<InsertRequest>,
    ) -> Result<Response<InsertResponse>, Status> {
    }

    async fn propose(
        &self,
        request: Request<ProposerRequest>,
    ) -> Result<Response<ProposerResponse>, Status> {
        let req = request.get_ref();

        let proposal = Proposal::new(
            req.seq_id.parse::<usize>().unwrap(),
            req.key.clone(),
            req.value.clone(),
        );

        let mut proposer = self.proposer.lock().await;

        let promise = proposer.prepare(proposal).unwrap();

        println!("{:?}", promise);

        //self.proposer.prepare(proposal)

        let reply = ProposerResponse {
            promised_proposal_id: promise.get_promised_proposal_id(),
            accepted_proposal_id: promise.get_accepted_proposal_id(),
            accepted_value: promise.get_accepted_value(),
        };

        Ok(Response::new(reply))
    }

    async fn accept(
        &self,
        request: Request<AcceptorRequest>,
    ) -> Result<Response<AcceptorResponse>, Status> {
        let req = request.get_ref();

        let mut proposer = self.proposer.lock().await;

        let proposal = Proposal::new(
            req.seq_id.parse::<usize>().unwrap(),
            req.key.clone(),
            req.value.clone(),
        );

        let m_proposal = ProposerRequest {
            seq_id: req.seq_id.clone(),
            key: req.key.clone(),
            value: req.value.clone(),
        };

        let accept = proposer.accept(proposal).unwrap();

        let reply = AcceptorResponse {
            status: 1 as i32,
            proposal_id: accept.proposal_id.to_string(),
            proposal: Some(m_proposal),
        };

        Ok(Response::new(reply))
    }

    async fn commit(
        &self,
        request: Request<LearnerRequest>,
    ) -> Result<Response<LearnerResponse>, Status> {
        let req = request.get_ref();
        let mut proposer = self.proposer.lock().await;

        let proposal = Proposal::new(
            req.proposal_id.parse::<usize>().unwrap(),
            req.key.clone(),
            req.value.clone(),
        );

        proposer.commit(proposal);
        let reply = LearnerResponse { status: true };
        Ok(Response::new(reply))
    }
}

impl NodeConfig {
    fn new(node_id: i32, addr: SocketAddr, status: Connection_Status) -> Self {
        Self {
            node_id,
            addr,
            status,
        }
    }
}
