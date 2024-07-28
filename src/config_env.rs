use crate::utils::rng_secret_key;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::{
    env::var,
    net::{IpAddr, Ipv4Addr},
};

#[derive(Serialize, Deserialize, Debug)]
pub struct NodeFileRecord {
    pub ip: IpAddr,
    pub tcp_port: u16,
    pub udp_port: u16,
    pub id: String,
}

lazy_static! {
    pub static ref PRIVATE_KEY: String = match var("PRIVATE_KEY") {
        Ok(value) => value.parse().unwrap(),
        Err(_) => {
            format!("{}", rng_secret_key().display_secret())
        }
    };
    pub static ref BOOT_NODE_IP: Option<IpAddr> = match var("BOOT_NODE_IP") {
        Ok(value) => Some(value.parse().unwrap()),
        Err(_) => {
            None
        }
    };
    pub static ref BOOT_NODE_TCP_PORT: Option<u16> = match var("BOOT_NODE_TCP_PORT") {
        Ok(value) => Some(value.parse().unwrap()),
        Err(_) => {
            None
        }
    };
    pub static ref BOOT_NODE_UDP_PORT: Option<u16> = match var("BOOT_NODE_UDP_PORT") {
        Ok(value) => Some(value.parse().unwrap()),
        Err(_) => {
            None
        }
    };
    pub static ref BOOT_NODE_ID: Option<String> = match var("BOOT_NODE_ID") {
        Ok(value) => Some(value.parse().unwrap()),
        Err(_) => {
            None
        }
    };
}
