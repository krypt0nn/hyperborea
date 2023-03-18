pub mod standards;

#[cfg(test)]
pub mod test;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Packet {
    #[cfg(feature = "packet-v1")]
    V1(standards::V1)
}

impl Packet {
    pub fn to_bytes(&self) -> Vec<u8> {
        #[allow(unreachable_patterns)]
        match self {
            #[cfg(feature = "packet-v1")]
            Self::V1(packet) => {
                let mut bytes = vec![0];

                bytes.append(&mut packet.to_bytes());

                bytes
            }

            _ => unreachable!()
        }
    }

    pub fn from_bytes<T: AsRef<[u8]>>(bytes: T) -> anyhow::Result<Self> {
        let bytes = bytes.as_ref();

        match bytes[0] {
            #[cfg(feature = "packet-v1")]
            0 => Ok(Self::V1(standards::V1::from_bytes(&bytes[1..])?)),

            _ => anyhow::bail!("Unsupported `packet::Packet` bytes sequence found: {:?}", bytes)
        }
    }
}

impl AsRef<Packet> for Packet {
    fn as_ref(&self) -> &Packet {
        self
    }
}

#[cfg(feature = "packet-v1")]
impl From<standards::V1> for Packet {
    #[inline]
    fn from(packet: standards::V1) -> Self {
        Self::V1(packet)
    }
}

#[cfg(feature = "packet-v1")]
impl PartialEq<standards::V1> for Packet {
    fn eq(&self, other: &standards::V1) -> bool {
        match self {
            Packet::V1(packet) => packet == other
        }
    }
}
