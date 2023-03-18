#[cfg(feature = "async")]
use tokio::net::UdpSocket;

#[cfg(not(feature = "async"))]
use std::net::UdpSocket;

use crate::node::Node;
use crate::node::owned::Node as OwnedNode;
use crate::packet::Packet;

#[cfg(test)]
pub mod test;

pub struct Controller {
    owned_node: OwnedNode,
    socket: UdpSocket
}

impl Controller {
    #[inline]
    #[cfg(not(feature = "async"))]
    pub fn new(owned_node: OwnedNode) -> anyhow::Result<Self> {
        Ok(Self {
            socket: UdpSocket::bind(owned_node.endpoint())?,
            owned_node
        })
    }

    #[inline]
    #[cfg(feature = "async")]
    pub async fn new(owned_node: OwnedNode) -> anyhow::Result<Self> {
        Ok(Self {
            socket: UdpSocket::bind(owned_node.endpoint()).await?,
            owned_node
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
    pub fn recv(&self) -> anyhow::Result<Packet> {
        let mut buf = [0; 1024];

        let len = self.socket.recv(&mut buf)?;

        Packet::from_bytes(&buf[..len])
    }

    #[cfg(feature = "async")]
    pub async fn recv(&self) -> anyhow::Result<Packet> {
        let mut buf = [0; 1024];

        let len = self.socket.recv(&mut buf).await?;

        Packet::from_bytes(&buf[..len])
    }
}
