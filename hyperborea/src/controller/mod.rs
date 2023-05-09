use std::sync::Arc;

use tokio::net::UdpSocket;

use crate::node::{Node, Standard as NodeStandard};
use crate::node::owned::{Node as OwnedNode, SignExt};

use crate::packet::Packet;
use crate::packet::standards as packets;

pub mod indexing;
pub mod requests;

mod standard;
mod params;

pub use standard::Standard;
pub use params::*;

#[cfg(test)]
pub mod test;

#[derive(Debug, Clone)]
pub struct Controller {
    owned_node: OwnedNode,
    socket: Arc<UdpSocket>,

    params: Params,
    storage: indexing::Storage,

    requests: requests::Requests
}

impl Controller {
    #[inline]
    pub async fn new(owned_node: OwnedNode, params: Params) -> anyhow::Result<Self> {
        Ok(Self {
            socket: Arc::new(UdpSocket::bind(owned_node.endpoint()).await?),
            owned_node,

            params,
            storage: params.indexing.strategy.into(),

            requests: requests::Requests::default()
        })
    }

    #[inline]
    pub fn owned_node(&self) -> &OwnedNode {
        &self.owned_node
    }

    #[inline]
    pub fn socket(&self) -> &UdpSocket {
        &self.socket
    }

    /// Send UDP packet to remote node
    pub async fn send<T: AsRef<Node>, F: AsRef<Packet>>(&self, node: T, packet: F) -> anyhow::Result<usize> {
        Ok(self.socket.send_to(
            &self.params.standard.to_bytes(node.as_ref(), packet, &self.owned_node)?,
            node.as_ref().endpoint()
        ).await?)
    }

    /// Receive UDP packet and try to decode it
    /// 
    /// Note that this method should not be used in end application.
    /// Use `update` method instead
    pub async fn recv(&self) -> anyhow::Result<(Node, Packet)> {
        let mut buf = [0; 65536];

        let (len, from) = self.socket.recv_from(&mut buf).await?;

        let (mut node, packet) = Standard::from_bytes(&buf[..len], &self.owned_node)?;

        if self.params.use_real_endpoint {
            node.address = from;
        }

        Ok((node, packet))
    }

    /// Update UDP socket
    pub async fn update(&mut self) -> anyhow::Result<()> {
        let (from, packet) = self.recv().await?;

        // Skip packet if our controller said to ignore v1 nodes
        match from.standard {
            NodeStandard::V1 { .. } => if !self.params.support_v1 {
                return Ok(());
            }
        }

        // Aggressive node indexing
        if self.params.indexing.aggressive {
            self.storage.insert(from.clone());
        }

        // Try to resolve outcoming request
        if let Some((_, response, _)) = self.requests.resolve(from.address(), &packet) {
            match response {
                // Remote has proved its availability (`AuthRequest`)
                requests::Response::AuthResponse(_) => {
                    self.storage.insert(from.clone());
                }
            }
        }

        // Or process incoming request
        else {
            match packet {
                Packet::V1(packet) => match packet {
                    packets::V1::AuthRequest(bytes) => {
                        let sign = self.owned_node.sign(bytes)?;

                        self.send::<_, Packet>(from, packets::V1::AuthResponse(sign).into()).await?;
                    }

                    packets::V1::Introduce(node) => {
                        // Naively index introduced node without verifying it
                        if self.params.indexing.naive {
                            self.storage.insert(node);
                        }

                        // Send node verification request
                        else {
                            let random_slice = [0; 1024].into_iter().map(|_| rand::random()).collect();

                            let packet: Packet = packets::Latest::AuthRequest(random_slice).into();

                            self.send(node.as_ref(), packet.as_ref()).await?;

                            self.requests.index(node.address(), packet);
                        }
                    }

                    // It was unregistered outcoming request
                    // or some random packet?
                    _ => ()
                }
            }
        }

        Ok(())
    }
}
