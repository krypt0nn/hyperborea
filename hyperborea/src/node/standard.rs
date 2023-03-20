use super::Address;

/// `node::Node` standard descriptor
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Standard {
    #[cfg(feature = "node-v1")]
    V1 {
        public_key: k256::PublicKey
    }
}

impl Standard {
    #[cfg(feature = "node-v1")]
    #[inline]
    pub fn latest(public_key: k256::PublicKey) -> Self {
        Self::V1 { public_key }
    }

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

impl From<Standard> for Address {
    #[inline]
    fn from(standard: Standard) -> Self {
        match standard {
            #[cfg(feature = "node-v1")]
            Standard::V1 { public_key } => public_key.into()
        }
    }
}

impl From<&Standard> for Address {
    #[inline]
    fn from(standard: &Standard) -> Self {
        match standard {
            #[cfg(feature = "node-v1")]
            Standard::V1 { public_key } => public_key.into()
        }
    }
}

impl AsRef<Standard> for Standard {
    #[inline]
    fn as_ref(&self) -> &Standard {
        self
    }
}
