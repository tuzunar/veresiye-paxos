use std::net::SocketAddr;
use std::sync::Arc;

use ::tokio::sync::Mutex;
use tonic::transport::Channel;

use crate::acceptor::Promise;
use crate::proposal;
use crate::proto::paxos_client::PaxosClient;
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

#[derive(Debug)]
pub struct NodeConfig {
    node_id: i32,
    addr: SocketAddr,
    status: Connection_Status,
}

#[derive(Debug)]
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
        println!(
            "register request arrived, new cluster status is {:?}",
            clusters
        );
        let reply = RegisterResponse {
            register_status: true,
        };

        Ok(Response::new(reply))
    }

    async fn send_register(
        &self,
        request: Request<RegisterRequest>,
    ) -> Result<Response<RegisterResponse>, Status> {
        let clusters = self.clusters.lock().await;
        let leader_id = self.leader_id.lock().await;
        let leader_node = clusters
            .iter()
            .find(|node| node.node_id == leader_id.unwrap())
            .unwrap();

        let leader_addr = format!("http://{}", leader_node.addr.clone());
        println!("leader addr is {}", leader_addr);
        let channel = Channel::from_shared(leader_addr.to_string())
            .unwrap()
            .connect()
            .await
            .unwrap();

        let mut client = PaxosClient::new(channel);

        let request = RegisterRequest {
            node_id: self.node_id.clone(),
            addr: self.addr.to_string().clone(),
            status: 0 as i32,
        };

        match client.register(request).await {
            Ok(response) => {
                println!("register operation successful {:?}", response);
            }
            Err(e) => {
                println!("register failed successful {}", e);
            }
        }

        let reply = RegisterResponse {
            register_status: true,
        };

        Ok(Response::new(reply))
    }

    /*
     * send propose all peers
     * calculate majority promise
     * if majority check failed increase proposal id and try agian(up to 3 times)
     * return the result
     */

    async fn insert(
        &self,
        request: Request<InsertRequest>,
    ) -> Result<Response<InsertResponse>, Status> {
        let req = request.get_ref();
        let clusters = self.clusters.clone();
        let node_id = self.node_id.clone();

        let mut proposer = self.proposer.lock().await;
        let old_proposer_id = proposer.get_last_seen_propose_id();
        println!("old propose id {}", old_proposer_id);
        proposer.set_last_seen_propose_id(old_proposer_id + 1 as i32);
        let propose_id = proposer.get_last_seen_propose_id();
        println!("new propose id {}", propose_id);
        let proposal = ProposerRequest {
            proposal_id: propose_id,
            key: req.key.clone(),
            value: req.value.clone(),
        };

        let mut accept: Vec<ProposerResponse> = vec![];

        let nodes = clusters.lock().await;
        println!("{:?}", nodes);
        for node in nodes.iter() {
            if node.node_id != node_id {
                println!("node id is {}", node.node_id.clone());
                let channel =
                    Channel::from_shared(format!("http://{}", node.addr.clone().to_string()))
                        .unwrap()
                        .connect()
                        .await
                        .unwrap();

                let mut client = PaxosClient::new(channel);
                let response = client.propose(proposal.clone());
                match response.await {
                    Ok(response) => {
                        let message = response.into_inner();
                        println!("response {:?}", message.clone());
                        accept.push(message.clone());
                        println!("request is here");
                    }
                    Err(e) => {}
                }
            }
        }
        println!("{} {} {}", accept.len(), nodes.len(), (nodes.len() / 2));
        if accept.len() >= nodes.len() / 2 {
            println!("majority reached");
            for message in accept {
                let node = nodes
                    .iter()
                    .find(|node| node.node_id == message.node_id)
                    .unwrap();

                let channel =
                    Channel::from_shared(format!("http://{}", node.addr.clone().to_string()))
                        .unwrap()
                        .connect()
                        .await
                        .unwrap();

                let mut client = PaxosClient::new(channel);
                if message.promised_proposal_id == proposal.clone().proposal_id {
                    let AcceptMessage = AcceptorRequest {
                        proposal_id: message.promised_proposal_id,
                        key: proposal.clone().key,
                        value: proposal.clone().value,
                    };

                    let accept = client.accept(AcceptMessage);

                    match accept.await {
                        Ok(response) => {
                            println!("accept is here");
                            let accept = response.into_inner();
                            println!(
                                "accept proposal {} and current proposal {}",
                                accept.proposal_id,
                                proposal.clone().proposal_id
                            );
                            if accept.proposal_id == proposal.clone().proposal_id {
                                let CommitMessage = LearnerRequest {
                                    proposal_id: accept.proposal_id,
                                    key: proposal.clone().key,
                                    value: proposal.clone().value,
                                };
                                client.commit(CommitMessage).await.unwrap();
                            }
                        }
                        Err(e) => {}
                    }
                }
            }
        } else {
            println!("majority not reached");
        }

        let reply = InsertResponse { result: true };

        Ok(Response::new(reply))
    }

    async fn propose(
        &self,
        request: Request<ProposerRequest>,
    ) -> Result<Response<ProposerResponse>, Status> {
        let req = request.get_ref();

        println!("incoming request {:?}", req.clone());

        let proposal = Proposal::new(req.proposal_id, req.key.clone(), req.value.clone());

        let mut proposer = self.proposer.lock().await;

        let promise = proposer.prepare(proposal).unwrap();

        println!("{:?}", promise);

        //self.proposer.prepare(proposal)

        let reply = ProposerResponse {
            node_id: self.node_id.clone(),
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

        let proposal = Proposal::new(req.proposal_id, req.key.clone(), req.value.clone());

        let m_proposal = ProposerRequest {
            proposal_id: req.proposal_id.clone(),
            key: req.key.clone(),
            value: req.value.clone(),
        };

        let accept = proposer.accept(proposal).unwrap();

        let reply = AcceptorResponse {
            node_id: self.node_id.clone(),
            status: 1 as i32,
            proposal_id: accept.proposal_id,
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

        let proposal = Proposal::new(req.proposal_id, req.key.clone(), req.value.clone());

        proposer.commit(proposal);
        let reply = LearnerResponse {
            node_id: self.node_id.clone(),
            status: true,
        };
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
