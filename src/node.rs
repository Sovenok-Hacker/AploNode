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
    outer_ip: IpAddr,
}

impl Node {
    pub fn new(
        blockchain: Arc<BlockChainTree>,
        timeout: Duration,
        stop: CancellationToken,
        outer_ip: IpAddr,
    ) -> Self {
        Self {
            blockchaintree: blockchain,
            tx_pool: Arc::new(RwLock::new(TxPool::new())),
            connected: Default::default(),
            stop,
            timeout,
            outer_ip,
        }
    }

    pub async fn start(self, mut peers_rx: UnboundedReceiver<SocketAddr>, port: u16) -> Result<()> {
        println!("Starting node");
        let self_cloned = self.clone();
        tokio::spawn(async move {
            let stop = self_cloned.stop.clone();
            loop {
                select! {
                    _ = stop.cancelled() => {
                        break;
                    }
                    peer = peers_rx.recv() => {
                        println!("Got propagated peer: {:?}", peer);
                        if let Some(peer) = peer {
                            if !self_cloned.connected.write().await.insert(peer) || peer.ip() == self.outer_ip{
                                continue;
                            }
                            let self_cloned = self_cloned.clone();
                            tokio::spawn((self_cloned.clone()).peer_connect(peer));
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
                    println!("New connection received: {:?}", peer);
                    let peer = peer?;

                    if !self.connected.write().await.insert(peer.1){
                        continue;
                    }

                    peer
                }
            };

            let self_cloned = self.clone();
            tokio::spawn(self_cloned.peer_handler(socket, address));
        }
        Ok(())
    }

    pub async fn peer_connect(self, address: SocketAddr) -> Result<()> {
        println!("Connecting to peer: {:?}", address);
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
                    println!("Recv timeouted");
                    break;
                }
                _ = stop.cancelled() => {
                    break;
                }
                packet = reader.next() => {
                    if let Some(Ok(packet)) = packet{
                        packet
                    }else{
                        println!("Error receiving packet {:?}", packet);
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
        //if !self.connected.write().await.insert(address) {
        //    println!("Peer: {} exists", address);
        //    return Ok(());
        //}

        println!("Established connection with: {}", address);
        let (read, write) = socket.into_split();
        let mut writer = FramedWrite::new(write, codec::PacketEncoder {});
        let reader = FramedRead::new(read, codec::PacketDecoder {});

        let (packet_tx, mut packet_rx) = unbounded_channel();

        let stop_token_clone = self.stop.clone();
        tokio::spawn(Node::read_packets(
            packet_tx,
            reader,
            self.timeout,
            stop_token_clone,
        ));

        writer
            .send(Packet::Request(Request::Ping { id: 0 }))
            .await?;
        loop {
            let packet = select! {
                _ = tokio::time::sleep(self.timeout / 2) => {
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
                        println!("Recieved ping request: {}", id);
                    }
                    Request::GetBlock() => todo!(),
                    Request::GetTransaction() => todo!(),
                    Request::GetBlockTransactions() => todo!(),
                    Request::LatestBlock() => todo!(),
                },
                Packet::Response(r) => match r {
                    Response::Ping { id } => println!("Got ping response: {}", id),
                    Response::GetBlock() => todo!(),
                    Response::GetTransaction() => todo!(),
                    Response::GetBlockTransactions() => todo!(),
                    Response::LatestBlock() => todo!(),
                },
            }
        }

        Ok(())
    }
}
