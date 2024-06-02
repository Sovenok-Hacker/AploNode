use crate::models;
use std::collections::HashSet;
use std::fs;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};
use tokio::fs as asyncfs;
use tokio::io::AsyncWriteExt;

use crate::errors::Errors;
use crate::manager::Manager;

pub struct Node {
    peers: Arc<RwLock<HashSet<SocketAddr>>>,
    domain_peers: Arc<RwLock<HashSet<String>>>,
    manager: Manager,
}

impl Node {
    pub fn new() -> Self {
        Node {
            peers: Default::default(),
            domain_peers: Default::default(),
            manager: Manager::new(),
        }
    }

    pub fn load_peers(&mut self, peers_file: String) -> Result<(), Errors> {
        let file_content = fs::read_to_string(peers_file)?;

        let peers: models::config::PeersFile = serde_json::from_str(&file_content)?;

        let mut peers_locked = self.peers.write().unwrap();
        for peer in peers.peers {
            peers_locked.insert(peer);
        }
        Ok(())
    }

    pub async fn store_peers(&self, peers_file: String) -> Result<(), Errors> {
        let peers_locked = self.peers.read().unwrap();
        let domain_peers = self.domain_peers.read().unwrap();
        let peers = serde_json::to_string(&models::config::PeersFile {
            peers: peers_locked.iter().cloned().collect(),
            domain_peers: domain_peers.iter().cloned().collect(),
        })?;
        drop(peers_locked);
        drop(domain_peers);

        let mut file = asyncfs::File::open(peers_file).await?;

        file.write_all(peers.as_bytes()).await?;

        Ok(())
    }
}
