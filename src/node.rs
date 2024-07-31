use std::net::{IpAddr, Ipv4Addr};
use std::time::Duration;
use std::{collections::HashSet, net::SocketAddr, sync::Arc};

use blockchaintree::blockchaintree::BlockChainTree;
use eyre::eyre;
use eyre::Result;
use std::sync::atomic::AtomicBool;
use tokio::net::{TcpListener, TcpStream};
use tokio::select;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;
use txpool::TxPool;

#[derive(Clone)]
pub struct Node {
    blockchaintree: Arc<BlockChainTree>,
    tx_pool: Arc<RwLock<TxPool>>,
    connected: Arc<RwLock<HashSet<SocketAddr>>>,
    stop: CancellationToken,
    timeout: Duration,
}

impl Node {
    pub fn new(blockchain: Arc<BlockChainTree>, timeout: Duration) -> Self {
        Self {
            blockchaintree: blockchain,
            tx_pool: Arc::new(RwLock::new(TxPool::new())),
            connected: Default::default(),
            stop: CancellationToken::new(),
            timeout,
        }
    }

    pub async fn start(self, mut peers_rx: UnboundedReceiver<SocketAddr>, port: u16) -> Result<()> {
        let self_cloned = self.clone();
        tokio::spawn(async move {
            let stop = self_cloned.stop.clone();
            loop {
                select! {
                    _ = stop.cancelled() => {
                        break;
                    }
                    peer = peers_rx.recv() => {
                        if let Some(peer) = peer {
                            if self_cloned.connected.read().await.get(&peer).is_some(){
                                continue;
                            }
                            let self_cloned = self_cloned.clone();
                            tokio::spawn(
                                async move { (self_cloned.clone()).peer_connect(peer) },
                            );
                        }else{
                            break;
                        }
                    }
                };
            }
        });

        let stream =
            TcpListener::bind(SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), port)).await?;

        loop {
            let (socket, address) = select! {
                _ = self.stop.cancelled() => {
                    break;
                }
                peer = stream.accept() => {

                    let peer = peer?;

                    if !self.connected.write().await.insert(peer.1){
                        continue;
                    }

                    peer
                }
            };

            let self_cloned = self.clone();
        }
        Ok(())
    }

    async fn peer_connect(self, address: SocketAddr) -> Result<()> {
        select! {
                _ = self.stop.cancelled() => {
                    Ok(())
                }
                stream = TcpStream::connect(address) => {

                    let stream = stream?;

                    tokio::spawn(async move {
                        // TODO: add proper error handling
                        let _ = self.peer_handler(stream, address).await;
                    });

                    Ok(())
                }
                _ = tokio::time::sleep(self.timeout) => {
                    Ok(())
        }
        }
    }

    pub async fn peer_handler(self, socket: TcpStream, address: SocketAddr) -> Result<()> {
        if !self.connected.write().await.insert(address) {
            return Ok(());
        }
        todo!()
    }
}
