use alloy_primitives::hex::FromHex as _;
use blockchaintree::blockchaintree::BlockChainTree;
use futures::StreamExt;
use node::Node;
use once_cell::sync::Lazy;
use reth_discv4::NodeRecord;

use alloy_primitives::B512;
use clap::Parser;
use reth_discv4::{DiscoveryUpdate, Discv4, Discv4ConfigBuilder, DEFAULT_DISCOVERY_ADDRESS};
use secp256k1::{generate_keypair, rand, SecretKey};
use std::collections::HashSet;
use std::fs;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::unbounded_channel;
use tokio_util::sync::CancellationToken;

use std::str::FromStr;

mod config_env;
mod utils;
use crate::config_env::*;
mod codec;
mod manager;
mod models;
mod node;

/// Aplo node
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Private key for the node, if nothing provided, random will be generated
    #[arg(short, long)]
    private_key: Option<String>,

    /// Path to the file with boot nodes
    #[arg(short, long, default_value = "boot_nodes.json")]
    boot_nodes: String,

    /// Path to the file with boot nodes
    #[arg(short, long, default_value = "33533")]
    udp_port: u16,

    /// Path to the file with boot nodes
    #[arg(short, long, default_value = "33533")]
    tcp_port: u16,

    /// Path to the file with boot nodes
    #[arg(short, long)]
    ip_address: String,
}

fn read_boot_nodes(args: &Args) -> Vec<NodeRecord> {
    let node_json: Vec<NodeFileRecord> =
        serde_json::from_str(&fs::read_to_string(&args.boot_nodes).unwrap()).unwrap();

    node_json
        .iter()
        .map(|node| NodeRecord {
            address: node.ip,
            tcp_port: node.tcp_port,
            udp_port: node.udp_port,
            id: B512::from_hex(&node.id).unwrap(),
        })
        .collect()
}

const ARGS: Lazy<Args> = Lazy::new(|| Args::parse());
static BOOT_NODES: Lazy<Vec<NodeRecord>> = Lazy::new(|| read_boot_nodes(&ARGS));
static BOOT_NODES_DICT: Lazy<HashSet<NodeRecord>> =
    Lazy::new(|| HashSet::from_iter(BOOT_NODES.iter().cloned()));

#[tokio::main]
async fn main() -> eyre::Result<()> {
    println!("loading blockchain");
    let blockchain = Arc::new(BlockChainTree::new("./BlockChainTree").unwrap());
    let mut stop = CancellationToken::new();
    println!("creating node");
    let node = Node::new(
        blockchain.clone(),
        Duration::from_secs(10),
        stop,
        ARGS.ip_address.parse().unwrap(),
    );

    let (peers_tx, peers_rx) = unbounded_channel::<SocketAddr>();

    tokio::spawn(node.start(peers_rx, ARGS.tcp_port));

    println!("{:?}", *BOOT_NODES);
    for node in BOOT_NODES.iter() {
        peers_tx
            .send(SocketAddr::new(node.address, node.tcp_port))
            .unwrap();
    }

    let secret_key = if let Some(private_key) = &ARGS.private_key {
        SecretKey::from_str(&private_key).unwrap()
    } else {
        generate_keypair(&mut rand::thread_rng()).0
    };
    let mut our_enr = NodeRecord::from_secret_key(
        SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, ARGS.udp_port)),
        &secret_key,
    );
    our_enr.tcp_port = ARGS.tcp_port;

    let mut discv4_cfg = Discv4ConfigBuilder::default();
    discv4_cfg
        .add_boot_nodes(BOOT_NODES.clone())
        .lookup_interval(Duration::from_secs(1));

    let discv4 = Discv4::spawn(our_enr.udp_addr(), our_enr, secret_key, discv4_cfg.build()).await?;
    let mut discv4_stream = discv4.update_stream().await?;

    while let Some(update) = discv4_stream.next().await {
        match update {
            DiscoveryUpdate::Added(peer) => {
                println!("New node: {:?}", peer);
                if BOOT_NODES_DICT.contains(&peer) {
                    continue;
                }

                if let Err(_) = peers_tx.send(SocketAddr::new(peer.address, peer.tcp_port)) {
                    break;
                }
            }
            DiscoveryUpdate::DiscoveredAtCapacity(_) => {}
            DiscoveryUpdate::EnrForkId(_, _) => {}
            DiscoveryUpdate::Removed(_) => {}
            DiscoveryUpdate::Batch(_) => {}
        }
    }

    println!("Stopped");

    Ok(())
}
