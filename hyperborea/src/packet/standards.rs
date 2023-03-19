use crate::node::Node;

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
    Introduce(Node)
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
        }
    }

    pub fn from_bytes<T: AsRef<[u8]>>(bytes: T) -> anyhow::Result<Self> {
        let bytes = bytes.as_ref();

        match bytes[0] {
            0 => Ok(Self::AuthRequest(bytes[1..].to_vec())),
            1 => Ok(Self::AuthResponse(bytes[1..].to_vec())),
            2 => Ok(Self::Introduce(Node::from_bytes(&bytes[1..])?)),

            _ => anyhow::bail!("Unsupported `packet::standards::V1` bytes sequence found: {:?}", bytes)
        }
    }
}
