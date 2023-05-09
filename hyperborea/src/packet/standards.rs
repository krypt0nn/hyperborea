use crate::node::{Node, Address};

#[cfg(feature = "packet-v1")]
pub type Latest = V1;

#[cfg(feature = "packet-v1")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum V1 {
    /// Request node verification (authentification)
    /// 
    /// Packet's param is a random data slice. If node works correctly -
    /// it'll answer with `AuthResponse` packet with signed param
    AuthRequest(Vec<u8>),

    /// Response to `AuthRequest` (signed original slice)
    AuthResponse(Vec<u8>),

    /// Used to introduce some node to another node
    /// 
    /// End node's behavior is not specified. One client can immediately register this node,
    /// another - request authentification, third - just ignore this packet
    Introduce(Node),

    /// Try to find node with given address
    SearchRequest {
        /// TTL (time to live) for this packet
        /// 
        /// This param specifies number of nodes it should visit.
        /// After each visit it decreases by 1, and if 0, then it will not be
        /// resent to other, deeper nodes in the network
        /// 
        /// Use `0` to request nodes only from remote you sent this packet to
        ttl: u8,

        /// Address of node we want to find in the network
        address: Address,

        /// Endpoint node `SearchResponse` should be sent to
        respond_to: Node
    },

    /// Response to `SearchRequest` with found node
    SearchResponse(Node)
}

#[cfg(feature = "packet-v1")]
impl V1 {
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            Self::AuthRequest(slice) => {
                let mut bytes = vec![0];

                bytes.extend_from_slice(slice);

                bytes
            }

            Self::AuthResponse(slice) => {
                let mut bytes = vec![1];

                bytes.extend_from_slice(slice);

                bytes
            }

            Self::Introduce(node) => {
                let mut bytes = vec![2];

                bytes.append(&mut node.to_bytes());

                bytes
            }

            Self::SearchRequest { ttl, address, respond_to } => {
                let mut bytes = vec![3, *ttl];

                let mut address = address.to_bytes();
                let mut respond_to = respond_to.to_bytes();

                bytes.extend_from_slice(&(address.len() as u16).to_be_bytes());
                bytes.append(&mut address);

                bytes.extend_from_slice(&(respond_to.len() as u16).to_be_bytes());
                bytes.append(&mut respond_to);

                bytes
            }

            Self::SearchResponse(node) => {
                let mut bytes = vec![4];

                bytes.append(&mut node.to_bytes());

                bytes
            }
        }
    }

    pub fn from_bytes<T: AsRef<[u8]>>(bytes: T) -> anyhow::Result<Self> {
        let bytes = bytes.as_ref();

        match bytes[0] {
            0 => Ok(Self::AuthRequest(bytes[1..].to_vec())),
            1 => Ok(Self::AuthResponse(bytes[1..].to_vec())),
            2 => Ok(Self::Introduce(Node::from_bytes(&bytes[1..])?)),
            4 => Ok(Self::SearchResponse(Node::from_bytes(&bytes[1..])?)),

            3 => {
                let ttl = bytes[1];

                let address_len = u16::from_be_bytes([bytes[2], bytes[3]]) as usize;
                let address = Address::from_bytes(&bytes[4..4 + address_len])?;

                let respond_to_len = u16::from_be_bytes([bytes[4 + address_len], bytes[5 + address_len]])  as usize;
                let respond_to = Node::from_bytes(&bytes[6 + address_len..6 + address_len + respond_to_len])?;

                Ok(Self::SearchRequest { ttl, address, respond_to })
            }

            _ => anyhow::bail!("Unsupported `packet::standards::V1` bytes sequence found: {:?}", bytes)
        }
    }
}