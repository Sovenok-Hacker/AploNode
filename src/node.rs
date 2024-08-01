use std::net::{IpAddr, Ipv4Addr};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::{collections::HashSet, net::SocketAddr, sync::Arc};

use crate::codec;
use crate::models::requests::Request;
use crate::models::responses::Response;
use crate::models::{requests, Packet};
use blockchaintree::blockchaintree::BlockChainTree;
use eyre::eyre;
use eyre::Result;
use futures::{SinkExt as _, StreamExt};
use std::sync::atomic::AtomicBool;
use tokio::net::tcp::{OwnedReadHalf, ReadHalf};
use tokio::net::{TcpListener, TcpStream};
use tokio::select;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::sync::RwLock;
use tokio_util::codec::{FramedRead, FramedWrite};
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
            tokio::spawn(async move { self_cloned.peer_handler(socket, address) });
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

    async fn read_packets(
        packets_sender: UnboundedSender<Packet>,
        mut reader: FramedRead<OwnedReadHalf, codec::PacketDecoder>,
        timeout: Duration,
        stop: CancellationToken,
    ) -> Result<()> {
        loop {
            let packet = select! {
                _ = tokio::time::sleep(timeout) => {
                    break;
                }
                _ = stop.cancelled() => {
                    break;
                }
                packet = reader.next() => {
                    if let Some(Ok(packet)) = packet{
                        packet
                    }else{
                        break;
                    }
                }
            };

            if let Err(_) = packets_sender.send(packet) {
                break;
            };
        }
        Ok(())
    }

    pub async fn peer_handler(self, socket: TcpStream, address: SocketAddr) -> Result<()> {
        if !self.connected.write().await.insert(address) {
            return Ok(());
        }

        let (read, write) = socket.into_split();
        let mut writer = FramedWrite::new(write, codec::PacketEncoder {});
        let reader = FramedRead::new(read, codec::PacketDecoder {});

        let (packet_tx, mut packet_rx) = unbounded_channel();

        let stop_token_clone = self.stop.clone();
        tokio::spawn(async move {
            Node::read_packets(packet_tx, reader, self.timeout, stop_token_clone)
        });

        writer
            .send(Packet::Request(Request::Ping { id: 0 }))
            .await?;
        loop {
            let packet = select! {
                _ = tokio::time::sleep(self.timeout) => {
                    writer
                        .send(Packet::Request(Request::Ping { id: 0 }))
                        .await?;
                    continue;
                }
                _ = self.stop.cancelled() => {
                    break;
                }
                packet = packet_rx.recv() => {
                    if let Some(packet) = packet {
                        packet
                    }else{
                        break;
                    }
                }
            };

            match packet {
                Packet::Request(r) => match r {
                    Request::Ping { id } => {
                        writer.send(Packet::Response(Response::Ping { id })).await?;
                        println!("Recieved ping packet, sending ping ack back");
                    }
                    Request::GetBlock() => todo!(),
                    Request::GetTransaction() => todo!(),
                    Request::GetBlockTransactions() => todo!(),
                    Request::LatestBlock() => todo!(),
                },
                Packet::Response(_) => todo!(),
            }
        }

        Ok(())
    }
}
