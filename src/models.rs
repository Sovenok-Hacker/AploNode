use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
pub struct Message {
    pub q: u8, // 0 - request, 1 - response
    #[serde(rename = "type")]
    pub body_type: u8,
    pub id: u32,
    pub body: rmpv::Value,
}

pub mod requests {
    use serde::{Deserialize, Serialize};

    #[derive(Deserialize, Serialize, Clone)]
    pub struct Ping {}
}

pub mod responses {

    use serde::{Deserialize, Serialize};

    #[derive(Deserialize, Serialize, Clone)]
    pub struct Pong {}
}

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
