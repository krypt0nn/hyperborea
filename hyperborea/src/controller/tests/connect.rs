use serial_test::serial;

use crate::node::owned::{
    Node as OwnedNode,
    Standard as OwnedStandard
};

use super::*;
use crate::controller::*;
use crate::packet::test::PACKETS;

use k256::SecretKey;

#[serial]
#[tokio::test]
async fn test_controller_connection() -> anyhow::Result<()> {
    let client = Controller::new(CLIENT_NODE.to_owned(), ControllerParams::default()).await?;
    let server = Controller::new(SERVER_NODE.to_owned(), ControllerParams::default()).await?;

    tokio::spawn(async move {
        for packet in PACKETS.iter() {
            assert_eq!(&server.recv().await.unwrap().1, packet);
        }
    });

    let server_node: crate::node::Node = SERVER_NODE.to_owned().into();

    for packet in PACKETS.iter().cloned() {
        client.send::<_, crate::packet::Packet>(&server_node, packet.into()).await?;
    }

    Ok(())
}

#[tokio::test]
async fn test_controller_mass_connection() -> anyhow::Result<()> {
    let node = OwnedNode::new(
        "127.0.0.1:50000".parse().unwrap(),
        OwnedStandard::latest(SecretKey::random(&mut rand::thread_rng()))
    );

    let server = Controller::new(node, ControllerParams::default()).await?;

    let server_node: crate::node::Node = server.owned_node().to_owned().into();

    let mut clients = Vec::with_capacity(1000);

    for i in 0..1000 {
        let node = OwnedNode::new(
            format!("127.0.0.1:{}", 50001 + i).parse().unwrap(),
            OwnedStandard::latest(SecretKey::random(&mut rand::thread_rng()))
        );

        if let Ok(controller) = Controller::new(node, ControllerParams::default()).await {
            clients.push(controller);
        }
    }

    assert!(clients.len() > 900);

    tokio::spawn(async move {
        for _ in 0..1000 {
            for packet in PACKETS.iter() {
                assert_eq!(&server.recv().await.unwrap().1, packet);
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
