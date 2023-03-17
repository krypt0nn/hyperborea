use std::net::{SocketAddr, Ipv4Addr, Ipv6Addr};

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Standard {
    V1
}

impl<'a> From<Standard> for &'a [u8] {
    fn from(standard: Standard) -> Self {
        match standard {
            Standard::V1 => &[0]
        }
    }
}

impl TryFrom<&[u8]> for Standard {
    type Error = anyhow::Error;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        match bytes[0] {
            0 => Ok(Self::V1),

            _ => anyhow::bail!("Unknown `node::Standard` bytes sequence found: {:?}", bytes)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Node {
    pub address: SocketAddr,
    pub standard: Standard
}

impl Node {
    
}

impl From<Node> for Vec<u8> {
    fn from(node: Node) -> Self {
        // IPv4 is the most used standard, 7 bytes
        // Standard V1 uses 1 byte
        let mut bytes = Vec::with_capacity(8);

        // Save endpoint address
        match node.address {
            // 7 bytes:
            // [0] [ip_1] [ip_2] [ip_3] [ip_4] [port_1] [port_2]
            SocketAddr::V4(addr) => {
                bytes.push(0);
                bytes.copy_from_slice(&addr.ip().octets());
                bytes.copy_from_slice(&addr.port().to_be_bytes());
            }

            // 11 bytes:
            // [0] [ip_1] [ip_2] [ip_3] [ip_4] [ip_5] [ip_6] [ip_7] [ip_8] [port_1] [port_2]
            SocketAddr::V6(addr) => {
                bytes.push(1);
                bytes.copy_from_slice(&addr.ip().octets());
                bytes.copy_from_slice(&addr.port().to_be_bytes());
            }
        }

        // Save protocol standard (1+ bytes)
        bytes.copy_from_slice(node.standard.into());

        bytes
    }
}

impl TryFrom<&[u8]> for Node {
    type Error = anyhow::Error;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        match bytes[0] {
            // IPv4
            0 => {
                let ip = Ipv4Addr::new(bytes[1], bytes[2], bytes[3], bytes[4]);
                let port = u16::from_be_bytes([bytes[5], bytes[6]]);

                Ok(Self {
                    address: SocketAddr::new(ip.into(), port),
                    standard: Standard::try_from(&bytes[7..])?
                })
            },

            // IPv6
            1 => {
                let ip = Ipv6Addr::from([
                    bytes[1],  bytes[2],  bytes[3],  bytes[4],
                    bytes[5],  bytes[6],  bytes[7],  bytes[8],
                    bytes[9],  bytes[10], bytes[11], bytes[12],
                    bytes[13], bytes[14], bytes[15], bytes[16]
                ]);

                let port = u16::from_be_bytes([bytes[17], bytes[18]]);

                Ok(Self {
                    address: SocketAddr::new(ip.into(), port),
                    standard: Standard::try_from(&bytes[19..])?
                })
            }

            _ => anyhow::bail!("Unknown `node::Node` address's bytes sequence found: {:?}", bytes)
        }
    }
}
