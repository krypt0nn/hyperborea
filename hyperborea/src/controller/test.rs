use std::net::SocketAddr;

use serial_test::serial;

use super::*;

use crate::node::owned::{
    Node as OwnedNode,
    Standard as OwnedStandard,
    VerifyExt
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

#[serial]
#[tokio::test]
async fn test_auth_request() -> anyhow::Result<()> {
    let client = Controller::new(CLIENT_NODE.to_owned(), ControllerParams::default()).await?;
    let mut server = Controller::new(SERVER_NODE.to_owned(), ControllerParams::default()).await?;

    let server_node_send: Node = server.owned_node().into();
    let server_node = server_node_send.clone();

    let random_slice_send = [0; 1024].into_iter().map(|_| rand::random()).collect::<Vec<u8>>();
    let random_slice = random_slice_send.clone();

    tokio::spawn(async move {
        loop {
            if let Err(err) = server.update().await {
                println!("Server error: {:?}", err);
            }
        }
    });

    let client_send = client.clone();

    let verifier = tokio::spawn(async move {
        let Ok((_, Packet::V1(packets::V1::AuthResponse(signed_slice)))) = client_send.recv().await else {
            anyhow::bail!("Wront response received");
        };

        server_node_send.verify(random_slice_send, signed_slice)
    });

    client.send::<_, Packet>(server_node, packets::V1::AuthRequest(random_slice).into()).await?;

    verifier.await?
}
