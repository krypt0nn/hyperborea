use crate::crypto::prelude::*;

use crate::time::timestamp;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ConnectionToken {
    /// Timestamp of the client connection request
    pub auth_date: u64,

    /// Public key of the server
    pub public_key: PublicKey
}

impl ConnectionToken {
    #[inline]
    pub fn new(public_key: PublicKey) -> Self {
        Self {
            auth_date: timestamp(),
            public_key
        }
    }

    pub fn to_bytes(&self) -> [u8; 41] {
        let mut certificate = [0u8; 41];

        certificate[..8].copy_from_slice(&self.auth_date.to_be_bytes());
        certificate[8..].copy_from_slice(&self.public_key.to_bytes());

        certificate
    }

    pub fn from_bytes(certificate: impl AsRef<[u8]>) -> Result<Self, CryptographyError> {
        let certificate = certificate.as_ref();

        let mut auth_date = [0u8; 8];

        auth_date.copy_from_slice(&certificate[..8]);

        Ok(Self {
            auth_date: u64::from_be_bytes(auth_date),
            public_key: PublicKey::from_bytes(&certificate[8..])?
        })
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    pub fn get_token() -> ConnectionToken {
        let public = SecretKey::random().public_key();

        ConnectionToken::new(public)
    }

    #[test]
    fn serialize() -> Result<(), CryptographyError> {
        let token = get_token();

        assert_eq!(ConnectionToken::from_bytes(token.to_bytes())?, token);

        Ok(())
    }
}
