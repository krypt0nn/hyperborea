use crate::crypto::prelude::*;

use crate::time::timestamp;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Information about the server and time when
/// the client connected to it.
/// 
/// This is a standard type declared in the
/// hyperborea protocol's paper.
pub struct ConnectionToken {
    /// Timestamp of the client connection request
    pub auth_date: u64,

    /// Public key of the server
    pub public_key: PublicKey
}

impl ConnectionToken {
    #[inline]
    /// Create new connection token.
    /// 
    /// - `auth_date` must contain UTC timestamp of the moment
    ///   when the client has connected to the server. Technically
    ///   this field can contain any value (although it can be
    ///   filtered by the servers), however it is recommended to
    ///   not to abuse it because otherwise your client would stick
    ///   with the given server forever and other servers will never
    ///   accept any updated info.
    /// 
    /// - `public_key` must contain public key of the server
    ///   the client has connected to.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use hyperborealib::crypto::prelude::*;
    /// use hyperborealib::rest_api::prelude::*;
    /// 
    /// use hyperborealib::time::timestamp;
    /// 
    /// let server_public = SecretKey::random().public_key();
    /// 
    /// let token = ConnectionToken::new(timestamp(), server_public);
    /// ```
    pub fn new(auth_date: u64, public_key: PublicKey) -> Self {
        Self {
            auth_date,
            public_key
        }
    }

    #[inline]
    /// Calls `new()` function with current timestamp.
    pub fn now(public_key: PublicKey) -> Self {
        Self::new(timestamp(), public_key)
    }

    /// Convert connection token to the fixed size
    /// bytes vector described in the protocol's standard.
    pub fn to_bytes(&self) -> [u8; 41] {
        let mut certificate = [0u8; 41];

        certificate[..8].copy_from_slice(&self.auth_date.to_be_bytes());
        certificate[8..].copy_from_slice(&self.public_key.to_bytes());

        certificate
    }

    /// Try to parse connection token from the given
    /// bytes vector.
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

        ConnectionToken::now(public)
    }

    #[test]
    fn serialize() -> Result<(), CryptographyError> {
        let token = get_token();

        assert_eq!(ConnectionToken::from_bytes(token.to_bytes())?, token);

        Ok(())
    }
}
