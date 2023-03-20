#[cfg(feature = "async")]
use tokio::net::UdpSocket;

#[cfg(not(feature = "async"))]
use std::net::UdpSocket;

use crate::node::Node;
use crate::node::owned::{Node as OwnedNode, SignExt};

use crate::packet::Packet;
use crate::packet::standards as packets;

pub mod indexing;

mod standard;

pub use standard::Standard;

#[cfg(test)]
pub mod test;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ControllerParams {
    /// Controller standard
    /// 
    /// Default is `Standard::default()`
    pub standard: Standard,

    /// Support V1 nodes standard
    /// 
    /// Default is `true`
    pub support_v1: bool,

    /// Add random useless bytes to the end of sending packets
    /// 
    /// This feature can somehow help to hide protocol detection
    /// 
    /// Default is `false`
    pub chaotic_tail: bool,

    /// Replace endpoint addresses in packet nodes
    /// by socket address which sent you this packet
    /// 
    /// Note that this can break compatibility with some systems
    /// 
    /// Default is `false`
    pub use_real_endpoint: bool,

    /// Algorithm used to store and share remote nodes
    /// 
    /// Default is `Strategy::default()`
    pub indexing_strategy: indexing::Strategy
}

impl Default for ControllerParams {
    fn default() -> Self {
        Self {
            standard: Standard::default(),
            support_v1: true,
            chaotic_tail: false,
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

    /// Send UDP packet to remote node
    #[cfg(not(feature = "async"))]
    pub fn send<T: AsRef<Node>, F: AsRef<Packet>>(&self, node: T, packet: F) -> anyhow::Result<usize> {
        self.socket.send_to(
            &self.params.standard.to_bytes(node.as_ref(), packet, &self.owned_node)?,
            node.as_ref().endpoint()
        )
    }

    /// Send UDP packet to remote node
    #[cfg(feature = "async")]
    pub async fn send<T: AsRef<Node>, F: AsRef<Packet>>(&self, node: T, packet: F) -> anyhow::Result<usize> {
        self.socket.send_to(
            &self.params.standard.to_bytes(node.as_ref(), packet, &self.owned_node)?,
            node.as_ref().endpoint()
        ).await.map_err(|e| e.into())
    }

    /// Receive UDP packet and try to decode it
    /// 
    /// Note that this method should not be used in end application.
    /// Use `update` method instead
    #[cfg(not(feature = "async"))]
    pub fn recv(&self) -> anyhow::Result<(Packet, Node)> {
        let mut buf = [0; 65536];

        let (len, from) = self.socket.recv_from(&mut buf)?;

        let (mut node, packet) = Standard::from_bytes(&buf[..len], &self.owned_node)?;

        if self.params.use_real_endpoint {
            node.address = from;
        }

        Ok((node, packet))
    }

    /// Receive UDP packet and try to decode it
    /// 
    /// Note that this method should not be used in end application.
    /// Use `update` method instead
    #[cfg(feature = "async")]
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
    #[cfg(feature = "async")]
    pub async fn update(&mut self) -> anyhow::Result<()> {
        let (from, packet) = self.recv().await?;

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
                    // TODO
                    self.storage.insert(node);
                }
            }
        }

        Ok(())
    }
}
