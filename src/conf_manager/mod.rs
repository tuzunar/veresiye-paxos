use std::{env::VarError, future};

use reqwest::{Error, Response};
use serde_json::json;
use tower::hedge::Future;

pub struct ConfigurationManager {
    eureka_address: String,
    eureka_port: u16,
    host_address: String,
    host_port: u16,
    node_id: i32,
    app_id: String,
}

impl ConfigurationManager {
    pub fn new(
        eureka_address: Result<String, VarError>,
        eureka_port: Result<String, VarError>,
        node_id: Result<String, VarError>,
        host_address: Result<String, VarError>,
        host_port: Result<String, VarError>,
        app_id: Result<String, VarError>,
    ) -> Self {
        let eureka_address = match eureka_address {
            Ok(value) => value,
            Err(e) => {
                println!("EUREKA_ADDR not provided using default(host.docker.internal) address");
                String::from("host.docker.internal")
            }
        };
        let eureka_port: u16 = match eureka_port {
            Ok(value) => value.parse().unwrap(),
            Err(e) => {
                println!("EUREKA_PORT not provided using default(8761) address");
                8176 as u16
            }
        };
        let node_id: i32 = match node_id {
            Ok(value) => value.parse().unwrap(),
            Err(e) => {
                println!("NODE_ID not provided using default(1) id");
                1 as i32
            }
        };
        let host_address = match host_address {
            Ok(value) => value.parse().unwrap(),
            Err(e) => {
                println!("HOST_ADDR not provided using default(host.docker.internal) address");
                String::from("host.docker.internal")
            }
        };

        let host_port: u16 = match host_port {
            Ok(value) => value.parse().unwrap(),
            Err(e) => {
                println!("HOST_PORT not provided using default(9000) id");
                9000 as u16
            }
        };

        let app_id = match app_id {
            Ok(value) => value,
            Err(e) => {
                println!("APP_ID not provided using default(veresiye) id");
                String::from("veresiye")
            }
        };
        Self {
            eureka_address,
            eureka_port,
            node_id,
            host_address,
            host_port,
            app_id,
        }
    }

    fn create_eureka_payload(&self, status: &str) -> serde_json::Value {
        //eureka registration body
        let payload = json!({
            "instance": {
                "instanceId": &self.node_id,
                "hostName": format!("veresiye-{}", &self.node_id),
                "app": "veresiye",
                "ipAddr": &self.host_address,
                "status": status,
                "port": {"$": &self.host_port, "@enabled": "true"},
                "securePort": {"$": 443, "@enabled": "false"},
                "homePageUrl": format!("http://localhost:{}", &self.host_port),
                "statusPageUrl": format!("http://localhost:{}", &self.host_port),
                "healthCheckUrl": format!("http://localhost:{}", &self.host_port),
                "dataCenterInfo": {
                    "@class": "com.netflix.appinfo.MyDataCenterInfo",
                    "name": "MyOwn"
                },
                "metadata": {
                    "leader": "true"
                }
            },
        });

        payload
    }

    fn get_eureka_address(&self) -> String {
        format!("http://{}:{}", &self.eureka_address, &self.eureka_port)
    }

    pub fn eureka_registration(&self) -> reqwest::RequestBuilder {
        let eureka_payload = &self.create_eureka_payload("STARTING");
        let eureka_address = &self.get_eureka_address();

        let req_client = reqwest::Client::new();

        req_client
            .post(format!(
                "{}/eureka/v2/apps/{}",
                eureka_address, &self.app_id
            ))
            .header("Accept", "application/json")
            .json(&eureka_payload)
    }

    pub fn eureka_heartbeat(&self) -> reqwest::RequestBuilder {
        let eureka_payload = &self.create_eureka_payload("UP");
        let eureka_address = &self.get_eureka_address();

        let req_client = reqwest::Client::new();

        req_client
            .post(format!(
                "{}/eureka/v2/apps/{}/{}",
                eureka_address, &self.app_id, &self.node_id
            ))
            .header("Accept", "application/json")
            .json(&eureka_payload)
    }

    pub fn get_host_port(&self) -> &u16 {
        &self.host_port
    }

    pub fn get_node_id(&self) -> &i32 {
        &self.node_id
    }
}
