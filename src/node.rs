use std::{collections::HashSet, net::SocketAddr, sync::Arc};

use blockchaintree::blockchaintree::BlockChainTree;
use eyre::Result;
use std::sync::atomic::AtomicBool;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::RwLock;
use txpool::TxPool;

use crate::manager::Manager;

#[derive(Clone)]
pub struct Node {
    blockchaintree: Arc<BlockChainTree>,
    tx_pool: Arc<RwLock<TxPool>>,
    connected: Arc<RwLock<HashSet<SocketAddr>>>,
    stop: Arc<AtomicBool>,
}

impl Node {
    pub fn new(blockchain: Arc<BlockChainTree>) -> Self {
        Self {
            blockchaintree: blockchain,
            tx_pool: Arc::new(RwLock::new(TxPool::new())),
            connected: Default::default(),
            stop: Arc::new(AtomicBool::new(false)),
        }
    }

    pub async fn start(self, peers_rx: UnboundedReceiver<SocketAddr>, port: u16) -> Result<()> {
        Ok(())
    }
}
