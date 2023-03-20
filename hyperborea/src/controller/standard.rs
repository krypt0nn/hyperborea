#[cfg(feature = "controller-v1")]
use aes_gcm::{KeyInit, aead::Aead};

use crate::node::Node;
use crate::node::owned::{Node as OwnedNode, SharedSecretExt};
use crate::packet::Packet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Standard {
    #[cfg(feature = "controller-v1")]
    V1
}

impl Default for Standard {
    #[inline]
    fn default() -> Self {
        #[cfg(feature = "controller-v1")]
        return Self::V1;

        #[cfg(not(feature = "controller-v1"))]
        unimplemented!()
    }
}

impl Standard {
    pub fn to_bytes<T, F, K>(&self, node: T, packet: F, sender: K) -> anyhow::Result<Vec<u8>>
    where
        T: AsRef<Node>,
        F: AsRef<Packet>,
        K: AsRef<OwnedNode>
    {
        #[allow(unreachable_patterns)]
        match self {
            #[cfg(feature = "controller-v1")]
            Self::V1 => {
                let mut buf = vec![0];

                let mut node_bytes = node.as_ref().to_bytes();
                let mut packet_bytes = packet.as_ref().to_bytes();

                // Append Node bytes
                buf.extend_from_slice(&(node_bytes.len() as u16).to_be_bytes());   // Node len (2 bytes)
                buf.append(&mut node_bytes);                                       // Node

                // Encrypt Packet bytes
                let mut secret = [0; 32];

                sender.as_ref().shared_secret::<_, &[u8]>(node.as_ref().standard, None, &mut secret)?;

                packet_bytes = aes_gcm::Aes256Gcm::new_from_slice(secret.as_slice())?
                    .encrypt(&aes_gcm::Nonce::default(), packet_bytes.as_slice())?;

                // Append Packet bytes
                buf.extend_from_slice(&(packet_bytes.len() as u16).to_be_bytes()); // Packet len (2 bytes)
                buf.append(&mut packet_bytes);                                     // Packet

                Ok(buf)
            }

            _ => unimplemented!()
        }
    }

    pub fn from_bytes<T: AsRef<[u8]>, F: AsRef<OwnedNode>>(bytes: T, receiver: F) -> anyhow::Result<(Node, Packet)> {
        let bytes = bytes.as_ref();

        match bytes[0] {
            #[cfg(feature = "controller-v1")]
            0 => {
                // Parse Node
                let node_len = u16::from_be_bytes([bytes[1], bytes[2]]) as usize;
                let node = Node::from_bytes(&bytes[3..3 + node_len])?;

                // Parse Packet
                let packet_len = u16::from_be_bytes([bytes[3 + node_len], bytes[4 + node_len]]) as usize;
                let packet = &bytes[5 + node_len..5 + node_len + packet_len];

                // Decrypt Packet bytes
                let mut secret = [0; 32];

                receiver.as_ref().shared_secret::<_, &[u8]>(node.as_ref().standard, None, &mut secret)?;

                let packet = aes_gcm::Aes256Gcm::new_from_slice(secret.as_slice())?
                    .decrypt(&aes_gcm::Nonce::default(), packet)?;

                Ok((node, Packet::from_bytes(packet)?))
            }

            _ => anyhow::bail!("Unsupported `controller::Standard` bytes sequence found: {:?}", bytes)
        }
    }
}
