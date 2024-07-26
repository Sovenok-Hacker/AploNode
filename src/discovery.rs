use crate::models;
use futures::{SinkExt as _, StreamExt as _};
use std::collections::HashSet;
use std::fs;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use tokio::fs as asyncfs;
use tokio::io::AsyncWriteExt;
use tokio::net::UdpSocket;
use tokio::sync::mpsc;
use tokio::sync::RwLock;
use tokio::time::timeout;
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use tokio_util::udp::UdpFramed;

use crate::errors::Errors;
use crate::manager::{Manager, MangerSender};

#[derive(Hash)]
pub struct NodeRecord {
    pub ip: IpAddr,
    pub udp_port: u16,
    pub tcp_port: u16,
}

pub struct NodeDiscovery {
    pub nodes: Arc<RwLock<HashSet<NodeRecord>>>,
    nodes_notifier: mpsc::UnboundedSender<NodeRecord>,
    announce_address: NodeRecord,
}

impl NodeDiscovery {
    pub fn new(
        announce_address: NodeRecord,
        nodes_notifier: mpsc::UnboundedSender<NodeRecord>,
    ) -> Self {
        NodeDiscovery {
            nodes: Default::default(),
            nodes_notifier,
            announce_address,
        }
    }

    pub async fn start(&mut self, listen_addr: IpAddr, boot_nodes: &[NodeRecord]) {
        let socket = UdpSocket::bind(SocketAddr::new(listen_addr, self.announce_address.udp_port))
            .await
            .unwrap();

        let stream = UdpFramed::new(socket, LengthDelimitedCodec::new());
    }

    async fn send(
        to: SocketAddr,
        message: models::discovery::DiscMessage,
        socket: &mut UdpFramed<LengthDelimitedCodec>,
    ) -> Result<(), std::io::Error> {
        socket
            .send((rmp_serde::to_vec(&message).unwrap().into(), to))
            .await
    }

    async fn send_peers(&self, peer_addr: SocketAddr, socket: Arc<UdpFramed<LengthDelimitedCodec>>) {
        for node in self.nodes.read().await.iter(){
            socket.
        }
    }

    //pub fn load_peers(&mut self, peers_file: String) -> Result<(), Errors> {
    //    let file_content = fs::read_to_string(peers_file)?;

    //    let peers: models::config::PeersFile = serde_json::from_str(&file_content)?;

    //    let mut peers_locked = self.peers.write();
    //    for peer in peers.peers {
    //        peers_locked.insert(peer);
    //    }
    //    Ok(())
    //}

    //pub async fn store_peers(&self, peers_file: String) -> Result<(), Errors> {
    //    let peers_locked = self.peers.read();
    //    let domain_peers = self.domain_peers.read();
    //    let peers = serde_json::to_string(&models::config::PeersFile {
    //        peers: peers_locked.iter().cloned().collect(),
    //        domain_peers: domain_peers.iter().cloned().collect(),
    //    })?;
    //    drop(peers_locked);
    //    drop(domain_peers);

    //    let mut file = asyncfs::File::open(peers_file).await?;

    //    file.write_all(peers.as_bytes()).await?;

    //    Ok(())
    //}
}
