use std::net::SocketAddr;

use super::*;

use crate::node::owned::{
    Node as OwnedNode,
    Standard as OwnedStandard
};

use crate::packet::test::PACKETS;

use k256::SecretKey;

lazy_static::lazy_static! {
    pub static ref CLIENT_ENDPOINT: SocketAddr = "127.0.0.1:49998".parse().unwrap();
    pub static ref SERVER_ENDPOINT: SocketAddr = "127.0.0.1:49999".parse().unwrap();

    pub static ref CLIENT_SECRET: SecretKey = SecretKey::random(&mut rand::thread_rng());
    pub static ref SERVER_SECRET: SecretKey = SecretKey::random(&mut rand::thread_rng());

    pub static ref CLIENT_NODE: OwnedNode = OwnedNode::new(*CLIENT_ENDPOINT, OwnedStandard::latest(CLIENT_SECRET.to_owned()));
    pub static ref SERVER_NODE: OwnedNode = OwnedNode::new(*SERVER_ENDPOINT, OwnedStandard::latest(SERVER_SECRET.to_owned()));
}

#[cfg(not(feature = "async"))]
#[test]
fn test_controller_connection() -> anyhow::Result<()> {
    let client = Controller::new(CLIENT_NODE.to_owned(), ControllerParams::default())?;
    let server = Controller::new(SERVER_NODE.to_owned(), ControllerParams::default())?;

    std::thread::spawn(move || {
        for packet in PACKETS.iter() {
            assert_eq!(&server.recv().unwrap().0, packet);
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
    let client = Controller::new(CLIENT_NODE.to_owned(), ControllerParams::default()).await?;
    let server = Controller::new(SERVER_NODE.to_owned(), ControllerParams::default()).await?;

    tokio::spawn(async move {
        for packet in PACKETS.iter() {
            assert_eq!(&server.recv().await.unwrap().0, packet);
        }
    });

    let server_node: crate::node::Node = SERVER_NODE.to_owned().into();

    for packet in PACKETS.iter().cloned() {
        client.send::<_, crate::packet::Packet>(&server_node, packet.into()).await?;
    }

    Ok(())
}

#[cfg(not(feature = "async"))]
#[test]
fn test_controller_mass_connection() -> anyhow::Result<()> {
    let node = OwnedNode::new(
        "127.0.0.1:50000".parse().unwrap(),
        OwnedStandard::latest(SecretKey::random(&mut rand::thread_rng()))
    );

    let server = Controller::new(node, ControllerParams::default())?;

    let server_node: crate::node::Node = server.owned_node().to_owned().into();

    let mut clients = Vec::with_capacity(10000);

    for i in 0..10000 {
        let node = OwnedNode::new(
            format!("127.0.0.1:{}", 50001 + i).parse().unwrap(),
            OwnedStandard::latest(SecretKey::random(&mut rand::thread_rng()))
        );

        if let Ok(controller) = Controller::new(node, ControllerParams::default())? {
            clients.push(controller);
        }
    }

    assert!(clients.len() > 9000);

    std::thread::spawn(move || {
        for _ in 0..10000 {
            for packet in PACKETS.iter() {
                assert_eq!(&server.recv().unwrap().0, packet);
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
    let node = OwnedNode::new(
        "127.0.0.1:50000".parse().unwrap(),
        OwnedStandard::latest(SecretKey::random(&mut rand::thread_rng()))
    );

    let server = Controller::new(node, ControllerParams::default()).await?;

    let server_node: crate::node::Node = server.owned_node().to_owned().into();

    let mut clients = Vec::with_capacity(10000);

    for i in 0..10000 {
        let node = OwnedNode::new(
            format!("127.0.0.1:{}", 50001 + i).parse().unwrap(),
            OwnedStandard::latest(SecretKey::random(&mut rand::thread_rng()))
        );

        if let Ok(controller) = Controller::new(node, ControllerParams::default()).await {
            clients.push(controller);
        }
    }

    assert!(clients.len() > 9000);

    tokio::spawn(async move {
        for _ in 0..10000 {
            for packet in PACKETS.iter() {
                assert_eq!(&server.recv().await.unwrap().0, packet);
            }
        }
    });

    for client in &clients {
        for packet in PACKETS.iter().cloned() {
            client.send::<_, crate::packet::Packet>(&server_node, packet.into()).await?;
        }
    }

    Ok(())
}
