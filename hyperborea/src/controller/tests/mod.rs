use std::net::SocketAddr;

use crate::node::owned::{
    Node as OwnedNode,
    Standard as OwnedStandard
};

use k256::SecretKey;

pub mod connect;
pub mod auth;
pub mod nodes_ring;

lazy_static::lazy_static! {
    pub static ref CLIENT_ENDPOINT: SocketAddr = "127.0.0.1:49998".parse().unwrap();
    pub static ref SERVER_ENDPOINT: SocketAddr = "127.0.0.1:49999".parse().unwrap();

    pub static ref CLIENT_SECRET: SecretKey = SecretKey::random(&mut rand::thread_rng());
    pub static ref SERVER_SECRET: SecretKey = SecretKey::random(&mut rand::thread_rng());

    pub static ref CLIENT_NODE: OwnedNode = OwnedNode::new(*CLIENT_ENDPOINT, OwnedStandard::latest(CLIENT_SECRET.to_owned()));
    pub static ref SERVER_NODE: OwnedNode = OwnedNode::new(*SERVER_ENDPOINT, OwnedStandard::latest(SERVER_SECRET.to_owned()));
}
