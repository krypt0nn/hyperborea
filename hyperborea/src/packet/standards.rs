#[cfg(feature = "packet-v1")]
pub type Latest = V1;

#[cfg(feature = "packet-v1")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum V1 {
    AuthRequest(u64),
    AuthResponse(u64)
}

#[cfg(feature = "packet-v1")]
impl V1 {
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            Self::AuthRequest(num) => {
                let mut bytes = vec![0];

                bytes.extend_from_slice(&num.to_be_bytes());

                bytes
            }

            Self::AuthResponse(num) => {
                let mut bytes = vec![1];

                bytes.extend_from_slice(&num.to_be_bytes());

                bytes
            }
        }
    }

    pub fn from_bytes<T: AsRef<[u8]>>(bytes: T) -> anyhow::Result<Self> {
        let bytes = bytes.as_ref();

        match bytes[0] {
            0 => Ok(Self::AuthRequest(u64::from_be_bytes([bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7], bytes[8]]))),
            1 => Ok(Self::AuthResponse(u64::from_be_bytes([bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7], bytes[8]]))),

            _ => anyhow::bail!("Unsupported `packet::standards::V1` bytes sequence found: {:?}", bytes)
        }
    }
}
