#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Address {
    #[cfg(feature = "node-v1")]
    V1(Vec<u8>)
}

impl Address {
    pub fn to_bytes(&self) -> Vec<u8> {
        #[allow(unreachable_patterns)]
        match self {
            #[cfg(feature = "node-v1")]
            Self::V1(address) => {
                let mut bytes = vec![0];

                bytes.extend_from_slice(address);

                bytes
            }

            _ => unreachable!()
        }
    }

    pub fn from_bytes<T: AsRef<[u8]>>(bytes: T) -> anyhow::Result<Self> {
        let bytes = bytes.as_ref();

        match bytes[0] {
            #[cfg(feature = "node-v1")]
            0 => Ok(Self::V1(bytes[1..].to_vec())),

            _ => anyhow::bail!("Unsupported `address::Address` bytes sequence found: {:?}", bytes)
        }
    }
}

impl std::fmt::Display for Address {
    #[allow(unused_variables, unreachable_patterns)]
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            #[cfg(feature = "node-v1")]
            // v1:08qn0a0vddm8e1rngljl5cmhsrdnoo8ej3la6m3m3k4v5j5euoh6i
            Self::V1(bytes) => write!(f, "v1:{}", data_encoding::BASE64URL_NOPAD.encode(bytes)),

            _ => unreachable!()
        }
    }
}

impl TryFrom<&str> for Address {
    type Error = anyhow::Error;

    fn try_from(address: &str) -> Result<Self, Self::Error> {
        #[cfg(feature = "node-v1")]
        if &address[..3] == "v1:" {
            let bytes = data_encoding::BASE64URL_NOPAD.decode(address[3..].as_bytes())?;

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

#[cfg(feature = "node-v1")]
impl From<&k256::PublicKey> for Address {
    #[inline]
    fn from(public_key: &k256::PublicKey) -> Self {
        Self::V1(public_key.to_sec1_bytes().to_vec())
    }
}

impl AsRef<Address> for Address {
    #[inline]
    fn as_ref(&self) -> &Self {
        self
    }
}
