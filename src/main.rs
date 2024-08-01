use alloy_primitives::hex::FromHex as _;
use blockchaintree::blockchaintree::BlockChainTree;
use futures::StreamExt;
use node::Node;
use once_cell::sync::Lazy;
use reth_discv4::NodeRecord;

use alloy_primitives::B512;
use clap::Parser;
use reth_discv4::{DiscoveryUpdate, Discv4, Discv4ConfigBuilder, DEFAULT_DISCOVERY_ADDRESS};
use secp256k1::SecretKey;
use std::collections::HashSet;
use std::fs;
use std::sync::Arc;
use std::time::Duration;

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
    #[arg(short, long, default_value = "boot.nodes")]
    boot_nodes: String,
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
    let blockchain = Arc::new(BlockChainTree::new("./BlockChainTree").unwrap());
    let node = Node::new(blockchain.clone(), Duration::from_secs(10));

    let our_key = SecretKey::from_str(&PRIVATE_KEY).unwrap();
    let our_enr = NodeRecord::from_secret_key(DEFAULT_DISCOVERY_ADDRESS, &our_key);

    let mut discv4_cfg = Discv4ConfigBuilder::default();
    discv4_cfg
        .add_boot_nodes(BOOT_NODES.clone())
        .lookup_interval(Duration::from_secs(1));

    let discv4 = Discv4::spawn(our_enr.udp_addr(), our_enr, our_key, discv4_cfg.build()).await?;
    let mut discv4_stream = discv4.update_stream().await?;

    while let Some(update) = discv4_stream.next().await {
        tokio::spawn(async move {
            if let DiscoveryUpdate::Added(peer) = update {
                println!("New node: {:?}", peer);
                if BOOT_NODES_DICT.contains(&peer) {
                    return;
                }
            }
        });
    }

    Ok(())
}
