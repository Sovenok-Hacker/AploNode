use alloy_primitives::hex::FromHex as _;
use futures::StreamExt;
use once_cell::sync::Lazy;
use reth_discv4::NodeRecord;

use alloy_primitives::B512;
use reth_discv4::{DiscoveryUpdate, Discv4, Discv4ConfigBuilder, DEFAULT_DISCOVERY_ADDRESS};
use secp256k1::SecretKey;
use std::time::Duration;

use std::str::FromStr;

mod config_env;
mod utils;
use crate::config_env::*;

fn mainnet_nodes_test() -> Vec<NodeRecord> {
    if BOOT_NODE_IP.is_some()
        && BOOT_NODE_TCP_PORT.is_some()
        && BOOT_NODE_ID.is_some()
        && BOOT_NODE_UDP_PORT.is_some()
    {
        vec![NodeRecord {
            address: BOOT_NODE_IP.unwrap(),
            tcp_port: BOOT_NODE_TCP_PORT.unwrap(),
            udp_port: BOOT_NODE_UDP_PORT.unwrap(),
            id: B512::from_hex(BOOT_NODE_ID.as_ref().unwrap()).unwrap(),
        }]
    } else {
        Vec::with_capacity(0)
    }
}

pub static MAINNET_BOOT_NODES: Lazy<Vec<NodeRecord>> = Lazy::new(mainnet_nodes_test);

#[tokio::main]
async fn main() -> eyre::Result<()> {
    print_config();
    let our_key = SecretKey::from_str(&PRIVATE_KEY).unwrap();
    let our_enr = NodeRecord::from_secret_key(DEFAULT_DISCOVERY_ADDRESS, &our_key);

    let mut discv4_cfg = Discv4ConfigBuilder::default();
    discv4_cfg
        .add_boot_nodes(MAINNET_BOOT_NODES.clone())
        .lookup_interval(Duration::from_secs(1));

    let discv4 = Discv4::spawn(our_enr.udp_addr(), our_enr, our_key, discv4_cfg.build()).await?;
    let mut discv4_stream = discv4.update_stream().await?;

    while let Some(update) = discv4_stream.next().await {
        tokio::spawn(async move {
            if let DiscoveryUpdate::Added(peer) = update {
                println!("New node: {:?}", peer);
                if MAINNET_BOOT_NODES.contains(&peer) {
                    return;
                }
            }
        });
    }

    Ok(())
}
