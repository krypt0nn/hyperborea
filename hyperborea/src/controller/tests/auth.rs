use serial_test::serial;

use super::*;
use crate::controller::*;
use crate::node::owned::VerifyExt;

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
