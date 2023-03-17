use std::net::{SocketAddr, Ipv4Addr, Ipv6Addr};

use serde::{Serialize, Deserialize};

#[cfg(test)]
mod test;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Standard {
    #[cfg(feature = "node-v1")]
    /// 1.0.0
    V1
}

impl Standard {
    #[inline]
    pub fn to_bytes(&self) -> &[u8] {
        match self {
            #[cfg(feature = "node-v1")]
            Standard::V1 => &[0],

            _ => unreachable!()
        }
    }

    #[inline]
    pub fn from_bytes<T: AsRef<[u8]>>(bytes: T) -> anyhow::Result<Self> {
        let bytes = bytes.as_ref();

        match bytes[0] {
            #[cfg(feature = "node-v1")]
            0 => Ok(Self::V1),

            _ => anyhow::bail!("Unsupported `node::Standard` bytes sequence found: {:?}", bytes)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Node {
    pub address: SocketAddr,
    pub standard: Standard
}

impl Node {
    #[inline]
    pub fn new(address: SocketAddr, standard: Standard) -> Self {
        Self {
            address,
            standard
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        // IPv4 is the most used standard, 7 bytes
        // Standard V1 uses 1 byte
        let mut bytes = Vec::with_capacity(8);

        // Save endpoint address
        match self.address {
            // 7 bytes:
            // [0] [ip_1] [ip_2] [ip_3] [ip_4] [port_1] [port_2]
            SocketAddr::V4(addr) => {
                bytes.push(0);
                bytes.extend_from_slice(&addr.ip().octets());
                bytes.extend_from_slice(&addr.port().to_be_bytes());
            }

            // 11 bytes:
            // [0] [ip_1] [ip_2] [ip_3] [ip_4] [ip_5] [ip_6] [ip_7] [ip_8] [port_1] [port_2]
            SocketAddr::V6(addr) => {
                bytes.push(1);
                bytes.extend_from_slice(&addr.ip().octets());
                bytes.extend_from_slice(&addr.port().to_be_bytes());
            }
        }

        // Save protocol standard (1+ bytes)
        bytes.extend_from_slice(self.standard.to_bytes());

        bytes
    }

    pub fn from_bytes<T: AsRef<[u8]>>(bytes: T) -> anyhow::Result<Self> {
        let bytes = bytes.as_ref();

        match bytes[0] {
            // IPv4
            0 => {
                let ip = Ipv4Addr::new(bytes[1], bytes[2], bytes[3], bytes[4]);
                let port = u16::from_be_bytes([bytes[5], bytes[6]]);

                Ok(Self {
                    address: SocketAddr::new(ip.into(), port),
                    standard: Standard::from_bytes(&bytes[7..])?
                })
            }

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
                    standard: Standard::from_bytes(&bytes[19..])?
                })
            }

            _ => anyhow::bail!("Unknown `node::Node` address's bytes sequence found: {:?}", bytes)
        }
    }
}
