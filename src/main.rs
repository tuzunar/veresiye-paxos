#![allow(warnings)]
mod proto {
    tonic::include_proto!("paxos");
    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("paxos_descriptor");
}
use conf_manager::ConfigurationManager;
use port_check::*;
use reqwest::header::HeaderMap;
use serde_json::json;
use std::env;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

use acceptor::{Accept_Message, Acceptor, Acceptor_Status, Promise};
use paxos::{NodeConfig, PaxosService};
use proposal::Proposal;
use proposer::Proposer;
use proto::paxos_server::{Paxos, PaxosServer};
use tonic::transport::{Channel, Server};
use tower_http::cors::CorsLayer;

mod acceptor;
mod conf_manager;
mod learner;
mod paxos;
mod proposal;
mod proposer;

/*
 *
 * -------------functions-------------
 * majority check
 * send prepare
 * prepare
 * send accept
 * accept
 * send commit
 * commit
 * send ping
 * ping
 *
 *
 * how should I store other instance's addresses
*/
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let conf_manager = ConfigurationManager::new(
        env::var("EUREKA_ADDR"),
        env::var("EUREKA_PORT"),
        env::var("NODE_ID"),
        env::var("HOST_ADDR"),
        env::var("HOST_PORT"),
        env::var("APP_ID"),
    );

    //register instance at starting
    match conf_manager.eureka_registration().send().await {
        Ok(resp) => println!("instance registered successfuly {:?}", resp),
        Err(e) => panic!("{e}"),
    }

    let addr = SocketAddr::new(
        IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
        *conf_manager.get_host_port(),
    );
    // let addr = "[::1]:54134".parse().unwrap();
    println!("Server is running on {}", addr);
    // paxos configuration
    let proposer = Arc::new(Mutex::new(Proposer::new(String::from("./data"))));
    let clusters: Arc<Mutex<Vec<NodeConfig>>> = Arc::new(Mutex::new(vec![]));
    let channels: Arc<Mutex<Vec<Channel>>> = Arc::new(Mutex::new(vec![]));
    let leader_id: Option<i32> = Some(1 as i32);
    let node_id: i32 = *conf_manager.get_node_id();
    // let leader_id: Option<i32> = None;
    let paxos = PaxosService::new(node_id, addr, clusters, channels, proposer, leader_id);

    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(proto::FILE_DESCRIPTOR_SET)
        .build_v1()
        .expect("cannot create reflection service");

    let layer = tower::ServiceBuilder::new()
        .layer(CorsLayer::new())
        .layer(tonic_web::GrpcWebLayer::new());

    Server::builder()
        .accept_http1(true)
        .layer(layer)
        .add_service(reflection_service)
        .add_service(PaxosServer::new(paxos))
        .serve(addr)
        .await?;
    Ok(())

    /*
    let mut paxos1 = Proposer::new(String::from("./data1"));
    let mut paxos2 = Proposer::new(String::from("./data2"));
    let mut paxos3 = Proposer::new(String::from("./data3"));
    let mut paxos4 = Proposer::new(String::from("./data4"));

    let mut multi_paxos: Vec<Proposer> = vec![paxos1, paxos2, paxos3, paxos4];

    let proposal = Proposal::new(2, String::from("key1"), String::from("value1"));

    let responses = send_prepare(&mut multi_paxos, proposal.clone());

    if responses.len() > (4 / 2) + 1 {
        println!("Majority Reached");
        println!("Sending Accept command");
        send_accept(&mut multi_paxos, proposal.clone());
    }
    */
}

fn send_prepare(multi_paxos: &mut Vec<Proposer>, proposal: Proposal) -> Vec<Promise> {
    println!("Sending prepare command each acceptor");
    let mut responses: Vec<Promise> = vec![];
    for paxos in multi_paxos.iter_mut() {
        match paxos.prepare(proposal.clone()) {
            Some(prom) => responses.push(prom),
            None => continue,
        }
    }

    responses
}

fn send_accept(multi_paxos: &mut Vec<Proposer>, proposal: Proposal) {
    println!("Sending accept command each acceptor");
    let mut responses: Vec<Accept_Message> = vec![];

    for paxos in multi_paxos.iter_mut() {
        match paxos.accept(proposal.clone()) {
            Some(message) => {
                if message.status == Acceptor_Status::Accepted {
                    paxos.commit(proposal.clone());
                }
            }
            None => eprintln!("Paxos return negative response"),
        }
    }
}

fn learner_commit() {}
