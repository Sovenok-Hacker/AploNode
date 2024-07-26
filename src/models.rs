use serde::{Deserialize, Serialize};
use serde_repr::*;

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

pub mod discovery {
    use std::net::SocketAddr;

    use super::*;
    use serde_repr::{Deserialize_repr, Serialize_repr};

    #[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug, Clone)]
    #[repr(u8)]
    pub enum DiscMessageType {
        RequestPeers,
        Ping,
        Announce,
    }

    #[derive(Deserialize, Serialize, Clone)]
    pub struct DiscMessage {
        pub r#type: DiscMessageType,
        pub version: u8,

        #[serde(with = "serde_bytes")]
        pub body: Vec<u8>,
    }

    #[derive(Deserialize, Serialize, Clone)]
    pub struct DiscAnnounce {
        pub address: SocketAddr,
        pub tcp_port: u16,
        pub udp_port: u16,
    }
}
