use std::net::{SocketAddr, Ipv4Addr, Ipv6Addr};

#[cfg(feature = "node-owned")]
pub mod owned;

mod address;
mod standard;

pub use address::Address;
pub use standard::Standard;

#[cfg(test)]
pub mod test;

#[derive(Debug, Clone, PartialEq, Eq)]
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

    #[inline]
    pub fn address(&self) -> Address {
        self.standard.into()
    }

    #[inline]
    pub fn endpoint(&self) -> SocketAddr {
        self.address
    }

    pub fn verify<T: AsRef<[u8]>>(&self, data: T, sign: T) -> anyhow::Result<()> {
        #[allow(unreachable_patterns)]
        match self.standard {
            #[cfg(feature = "node-v1")]
            Standard::V1 { public_key } => {
                use k256::ecdsa::signature::Verifier;

                let sign = k256::ecdsa::Signature::from_der(sign.as_ref())?;

                k256::ecdsa::VerifyingKey::from_affine(*public_key.as_affine())?
                    .verify(data.as_ref(), &sign)
                    .map_err(|e| e.into())
            }

            _ => unreachable!()
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

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

        // Save protocol standard
        bytes.append(&mut self.standard.to_bytes());

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

impl AsRef<Node> for Node {
    fn as_ref(&self) -> &Node {
        self
    }
}
