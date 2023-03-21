#[cfg(feature = "async")]
use tokio::net::UdpSocket;

#[cfg(not(feature = "async"))]
use std::net::UdpSocket;

use crate::node::Node;
use crate::node::owned::{Node as OwnedNode, SignExt};

use crate::packet::Packet;
use crate::packet::standards as packets;

pub mod indexing;
pub mod requests;

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
    pub indexing_strategy: indexing::Strategy,

    /// Use naive indexing strategy (always index `Introduce` nodes)
    /// without verifying them using `AuthRequest` packet
    /// 
    /// This feature can be abused by malicious nodes, but it also
    /// significantly reduces amount of sent UDP packets
    /// 
    /// Default is `false`
    pub naive_indexing: bool,

    /// Index nodes from every incoming packet
    /// 
    /// This feature won't verify indexing nodes using `AuthRequest` packet
    /// 
    /// Default is `false`
    pub aggressive_indexing: bool
}

impl Default for ControllerParams {
    fn default() -> Self {
        Self {
            standard: Standard::default(),
            support_v1: true,
            use_real_endpoint: false,
            indexing_strategy: indexing::Strategy::default(),
            naive_indexing: false,
            aggressive_indexing: false
        }
    }
}

#[derive(Debug)]
pub struct Controller {
    owned_node: OwnedNode,
    socket: UdpSocket,

    params: ControllerParams,
    storage: indexing::Storage,

    requests: requests::Requests
}

impl Controller {
    #[inline]
    #[cfg(not(feature = "async"))]
    pub fn new(owned_node: OwnedNode, params: ControllerParams) -> anyhow::Result<Self> {
        Ok(Self {
            socket: UdpSocket::bind(owned_node.endpoint())?,
            owned_node,

            params,
            storage: params.indexing_strategy.into(),

            requests: requests::Requests::default()
        })
    }

    #[inline]
    #[cfg(feature = "async")]
    pub async fn new(owned_node: OwnedNode, params: ControllerParams) -> anyhow::Result<Self> {
        Ok(Self {
            socket: UdpSocket::bind(owned_node.endpoint()).await?,
            owned_node,

            params,
            storage: params.indexing_strategy.into(),

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

        // Aggressive node indexing
        if self.params.aggressive_indexing {
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
                        if self.params.naive_indexing {
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
