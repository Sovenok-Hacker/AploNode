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

pub mod peers {
    use serde::{Deserialize, Serialize};
    use std::net::SocketAddr;

    #[derive(Deserialize, Serialize, Clone)]
    pub struct PeersFile {
        pub peers: Vec<SocketAddr>,
    }
}
