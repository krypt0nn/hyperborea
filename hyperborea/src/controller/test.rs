use std::net::SocketAddr;

use super::*;

use crate::node::owned::{
    Node as OwnedNode,
    Standard as OwnedStandard
};

use crate::packet::test::PACKETS;

use k256::SecretKey;

lazy_static::lazy_static! {
    static ref CLIENT_ENDPOINT: SocketAddr = "127.0.0.1:49998".parse().unwrap();
    static ref SERVER_ENDPOINT: SocketAddr = "127.0.0.1:49999".parse().unwrap();

    static ref CLIENT_SECRET: SecretKey = SecretKey::random(&mut rand::thread_rng());
    static ref SERVER_SECRET: SecretKey = SecretKey::random(&mut rand::thread_rng());

    static ref CLIENT_NODE: OwnedNode = OwnedNode::new(*CLIENT_ENDPOINT, OwnedStandard::latest(CLIENT_SECRET.to_owned()));
    static ref SERVER_NODE: OwnedNode = OwnedNode::new(*SERVER_ENDPOINT, OwnedStandard::latest(SERVER_SECRET.to_owned()));
}

#[cfg(not(feature = "async"))]
#[test]
fn test_controller_connection() -> anyhow::Result<()> {
    let client = Controller::new(CLIENT_NODE.to_owned())?;
    let server = Controller::new(SERVER_NODE.to_owned())?;

    std::thread::spawn(move || {
        for packet in PACKETS.iter() {
            assert_eq!(&server.recv().unwrap(), packet);
        }
    });

    let server_node: crate::node::Node = SERVER_NODE.to_owned().into();

    for packet in PACKETS.iter().copied() {
        client.send::<_, crate::packet::Packet>(&server_node, packet.into())?;
    }

    Ok(())
}

#[cfg(feature = "async")]
#[tokio::test]
async fn test_controller_connection_async() -> anyhow::Result<()> {
    let client = Controller::new(CLIENT_NODE.to_owned()).await?;
    let server = Controller::new(SERVER_NODE.to_owned()).await?;

    tokio::spawn(async move {
        for packet in PACKETS.iter() {
            assert_eq!(&server.recv().await.unwrap(), packet);
        }
    });

    let server_node: crate::node::Node = SERVER_NODE.to_owned().into();

    for packet in PACKETS.iter().copied() {
        client.send::<_, crate::packet::Packet>(&server_node, packet.into()).await?;
    }

    Ok(())
}

#[cfg(not(feature = "async"))]
#[test]
fn test_controller_mass_connection() -> anyhow::Result<()> {
    let server = Controller::new(OwnedNode::new(
        "127.0.0.1:50000".parse().unwrap(),
        OwnedStandard::latest(SecretKey::random(&mut rand::thread_rng()))
    ))?;

    let server_node: crate::node::Node = server.owned_node().to_owned().into();

    let mut clients = Vec::with_capacity(10000);

    for i in 0..10000 {
        clients.push(Controller::new(OwnedNode::new(
            format!("127.0.0.1:{}", 50001 + i).parse().unwrap(),
            OwnedStandard::latest(SecretKey::random(&mut rand::thread_rng()))
        ))?);
    }

    std::thread::spawn(move || {
        for _ in 0..10000 {
            for packet in PACKETS.iter() {
                assert_eq!(&server.recv().unwrap(), packet);
            }
        }
    });

    for client in clients {
        for packet in PACKETS.iter().copied() {
            client.send::<_, crate::packet::Packet>(&server_node, packet.into())?;
        }
    }

    Ok(())
}

#[cfg(feature = "async")]
#[tokio::test]
async fn test_controller_mass_connection_async() -> anyhow::Result<()> {
    let server = Controller::new(OwnedNode::new(
        "127.0.0.1:50000".parse().unwrap(),
        OwnedStandard::latest(SecretKey::random(&mut rand::thread_rng()))
    )).await?;

    let server_node: crate::node::Node = server.owned_node().to_owned().into();

    let mut clients = Vec::with_capacity(10000);

    for i in 0..10000 {
        clients.push(Controller::new(OwnedNode::new(
            format!("127.0.0.1:{}", 50001 + i).parse().unwrap(),
            OwnedStandard::latest(SecretKey::random(&mut rand::thread_rng()))
        )).await?);
    }

    tokio::spawn(async move {
        for _ in 0..10000 {
            for packet in PACKETS.iter() {
                assert_eq!(&server.recv().await.unwrap(), packet);
            }
        }
    });

    for client in &clients {
        for packet in PACKETS.iter().copied() {
            client.send::<_, crate::packet::Packet>(&server_node, packet.into()).await?;
        }
    }

    Ok(())
}
