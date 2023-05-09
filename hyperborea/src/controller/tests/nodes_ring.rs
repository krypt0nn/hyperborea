use serial_test::serial;

use super::*;
use crate::node::owned::Node as OwnedNode;
use crate::controller::*;

#[serial]
#[tokio::test]
async fn test_nodes_ring() -> anyhow::Result<()> {
    let mut server = Controller::new(SERVER_NODE.clone(), Params::default()).await?;

    let mut ring = Vec::<(OwnedNode, _)>::with_capacity(1000);

    for i in 0..1000 {
        let node = OwnedNode::new(
            format!("127.0.0.1:{}", 50000 + i).parse().unwrap(),
            OwnedStandard::latest(SecretKey::random(&mut rand::thread_rng()))
        );

        if let Ok(mut controller) = Controller::new(node.clone(), Params::default()).await {
            if let Some((last, _)) = ring.last() {
                controller.storage.insert(last.into());
            }

            ring.push((node, tokio::spawn(async move {
                loop {
                    if let Err(err) = controller.update().await {
                        println!("Controller #{i} failed: {err}");

                        break;
                    }
                }
            })));
        }
    }

    if let Some((last, _)) = ring.last() {
        server.storage.insert(last.into());

        server.find_remote(ring.first().unwrap().0.address()).await;
    }

    Ok(())
}
