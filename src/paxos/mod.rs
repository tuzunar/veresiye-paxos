use crate::{acceptor::Acceptor, learner::Learner, proposal::Proposal, proposer::Proposer};
use proto::{
    paxos_service_server::{PaxosService, PaxosServiceServer},
    AcceptorRequest, AcceptorResponse, LearnerRequest, LearnerResponse, PingRequest, PingResponse,
    ProposerRequest, ProposerResponse,
};
use tonic::{Request, Response, Status};

mod proto {
    tonic::include_proto!("paxos");
}

#[derive(Default)]
struct Paxos {
    propser: Proposer,
    proposal: Proposal,
    acceptor: Acceptor,
    learner: Learner,
}

#[tonic::async_trait]
impl PaxosService for Paxos {
    async fn ping(&self, request: Request<PingRequest>) -> Result<Response<PingResponse>, Status> {}
    async fn propose(
        &self,
        request: Request<ProposerRequest>,
    ) -> Result<Response<ProposerResponse>, Status> {
    }

    async fn accept(
        &self,
        request: Request<AcceptorRequest>,
    ) -> Result<Response<AcceptorResponse>, Status> {
    }

    async fn commit(
        &self,
        request: Request<LearnerRequest>,
    ) -> Result<Response<LearnerResponse>, Status> {
    }
}
