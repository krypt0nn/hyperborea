use std::net::SocketAddr;

#[cfg(feature = "async")]
use tokio::net::UdpSocket;

#[cfg(not(feature = "async"))]
use std::net::UdpSocket;

use crate::node::Node;
use crate::node::owned::Node as OwnedNode;
use crate::node::owned::SignExt;

use crate::packet::Packet;
use crate::packet::standards as packets;

pub mod indexing;

#[cfg(test)]
pub mod test;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ControllerParams {
    /// Support V1 nodes standard
    /// 
    /// `true` by default
    pub support_v1: bool,

    /// Replace endpoint addresses in `Packet::Introduce` nodes
    /// by socket address which sent you this packet
    /// 
    /// Note that this can break compatibility with some systems
    /// 
    /// `false` by default
    pub use_real_endpoint: bool,

    /// Algorithm used to store and share remote nodes
    /// 
    /// `Storage(None)` by default
    pub indexing_strategy: indexing::Strategy
}

impl Default for ControllerParams {
    fn default() -> Self {
        Self {
            support_v1: true,
            use_real_endpoint: false,
            indexing_strategy: indexing::Strategy::default()
        }
    }
}

#[derive(Debug)]
pub struct Controller {
    owned_node: OwnedNode,
    socket: UdpSocket,

    params: ControllerParams,
    storage: indexing::Storage
}

impl Controller {
    #[inline]
    #[cfg(not(feature = "async"))]
    pub fn new(owned_node: OwnedNode, params: ControllerParams) -> anyhow::Result<Self> {
        Ok(Self {
            socket: UdpSocket::bind(owned_node.endpoint())?,
            owned_node,
            params,
            storage: params.indexing_strategy.into()
        })
    }

    #[inline]
    #[cfg(feature = "async")]
    pub async fn new(owned_node: OwnedNode, params: ControllerParams) -> anyhow::Result<Self> {
        Ok(Self {
            socket: UdpSocket::bind(owned_node.endpoint()).await?,
            owned_node,
            params,
            storage: params.indexing_strategy.into()
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

    #[cfg(not(feature = "async"))]
    pub fn send<T: AsRef<Node>, F: AsRef<Packet>>(&self, node: T, packet: F) -> std::io::Result<usize> {
        self.socket.send_to(&packet.as_ref().to_bytes(), node.as_ref().endpoint())
    }

    #[cfg(feature = "async")]
    pub async fn send<T: AsRef<Node>, F: AsRef<Packet>>(&self, node: T, packet: F) -> std::io::Result<usize> {
        self.socket.send_to(&packet.as_ref().to_bytes(), node.as_ref().endpoint()).await
    }

    #[cfg(not(feature = "async"))]
    pub fn recv(&self) -> anyhow::Result<(Packet, SocketAddr)> {
        let mut buf = [0; 1024];

        let (len, from) = self.socket.recv_from(&mut buf)?;

        Ok((Packet::from_bytes(&buf[..len])?, from))
    }

    /// Receive UDP packet and try to decode it
    /// 
    /// Note that this method should not be used in end application.
    /// Use `update` method instead
    #[cfg(feature = "async")]
    pub async fn recv(&self) -> anyhow::Result<(Packet, SocketAddr)> {
        let mut buf = [0; 1024];

        let (len, from) = self.socket.recv_from(&mut buf).await?;

        Ok((Packet::from_bytes(&buf[..len])?, from))
    }

    /// Update UDP socket
    #[cfg(feature = "async")]
    pub async fn update(&mut self) -> anyhow::Result<()> {
        let (packet, from) = self.recv().await?;

        match packet {
            Packet::V1(packet) => match packet {
                packets::V1::AuthRequest(bytes) => {
                    let sign = self.owned_node.sign(bytes)?;

                    // self.send(, packet)
                }

                packets::V1::AuthResponse(bytes) => {
                    // TODO
                }

                packets::V1::Introduce(mut node) => {
                    if self.params.use_real_endpoint {
                        node.address = from;
                    }

                    // TODO
                    self.storage.insert(node);
                }
            }
        }

        Ok(())
    }
}
