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
