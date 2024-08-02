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
