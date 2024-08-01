use serde::{Deserialize, Serialize};
use serde_repr::*;

use self::responses::Response;

pub mod config {
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;
    use std::net::SocketAddr;
    use std::path::PathBuf;

    #[derive(Deserialize, Serialize, Clone)]
    pub struct PeersFile {
        pub peers: Vec<SocketAddr>,
        pub domain_peers: Vec<String>,
    }

    #[inline]
    fn default_logs() -> PathBuf {
        PathBuf::from("./logs")
    }

    #[derive(Deserialize, Serialize, Clone, Debug)]
    pub struct LogConfig {
        pub global_directive: Option<String>,
        pub directives: Option<HashMap<String, String>>,

        #[serde(default = "default_logs")]
        pub logs_dir: PathBuf,
    }

    #[derive(Deserialize, Serialize, Clone, Debug)]
    pub struct Config {
        pub server_addr: SocketAddr,
        pub log_config: LogConfig,
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(tag = "q", content = "data")]
pub enum Packet {
    #[serde(rename = "t")]
    Request(requests::Request),

    #[serde(rename = "r")]
    Response(Response),
}

pub mod requests {
    use super::*;

    #[derive(Deserialize, Serialize, Clone, Debug)]
    #[serde(tag = "t", content = "data")]
    pub enum Request {
        #[serde(rename = "1")]
        Ping { id: u32 },
        #[serde(rename = "2")]
        GetBlock(),
        #[serde(rename = "3")]
        GetTransaction(),
        #[serde(rename = "4")]
        GetBlockTransactions(),
        #[serde(rename = "5")]
        LatestBlock(),
    }
}

pub mod responses {
    use super::*;

    #[derive(Deserialize, Serialize, Clone, Debug)]
    #[serde(tag = "t", content = "data")]
    pub enum Response {
        #[serde(rename = "1")]
        Ping { id: u32 },
        #[serde(rename = "2")]
        GetBlock(),
        #[serde(rename = "3")]
        GetTransaction(),
        #[serde(rename = "4")]
        GetBlockTransactions(),
        #[serde(rename = "5")]
        LatestBlock(),
    }
}
