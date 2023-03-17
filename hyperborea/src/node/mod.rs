use std::net::{SocketAddr, Ipv4Addr, Ipv6Addr};

#[cfg(test)]
mod test;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Address {
    #[cfg(feature = "node-v1")]
    V1(Vec<u8>)
}

impl std::fmt::Display for Address {
    #[allow(unused_variables, unreachable_patterns)]
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            #[cfg(feature = "node-v1")]
            // v1:08qn0a0vddm8e1rngljl5cmhsrdnoo8ej3la6m3m3k4v5j5euoh6i
            Self::V1(bytes) => write!(f, "v1:{}", data_encoding::BASE32_DNSSEC.encode(bytes)),

            _ => unreachable!()
        }
    }
}

impl TryFrom<&str> for Address {
    type Error = anyhow::Error;

    fn try_from(address: &str) -> Result<Self, Self::Error> {
        #[cfg(feature = "node-v1")]
        if &address[..3] == "v1:" {
            let bytes = data_encoding::BASE32_DNSSEC.decode(address[3..].as_bytes())?;

            if k256::PublicKey::from_sec1_bytes(&bytes).is_ok() {
                return Ok(Self::V1(bytes))
            }
        }

        anyhow::bail!("Unable to decode address: {address}");
    }
} 

#[cfg(feature = "node-v1")]
impl From<k256::PublicKey> for Address {
    #[inline]
    fn from(public_key: k256::PublicKey) -> Self {
        Self::V1(public_key.to_sec1_bytes().to_vec())
    }
}

impl From<Standard> for Address {
    #[inline]
    fn from(standard: Standard) -> Self {
        match standard {
            #[cfg(feature = "node-v1")]
            Standard::V1 { public_key } => public_key.into()
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Standard {
    #[cfg(feature = "node-v1")]
    V1 {
        public_key: k256::PublicKey
    }
}

impl Standard {
    #[inline]
    pub fn to_bytes(&self) -> Vec<u8> {
        #[allow(unreachable_patterns)]
        match self {
            #[cfg(feature = "node-v1")]
            Standard::V1 { public_key } => {
                let mut bytes = vec![0];

                bytes.extend_from_slice(&public_key.to_sec1_bytes());

                bytes
            },

            _ => unreachable!()
        }
    }

    #[inline]
    pub fn from_bytes<T: AsRef<[u8]>>(bytes: T) -> anyhow::Result<Self> {
        let bytes = bytes.as_ref();

        match bytes[0] {
            #[cfg(feature = "node-v1")]
            0 => Ok(Self::V1 {
                public_key: k256::PublicKey::from_sec1_bytes(&bytes[1..])?
            }),

            _ => anyhow::bail!("Unsupported `node::Standard` bytes sequence found: {:?}", bytes)
        }
    }
}

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
