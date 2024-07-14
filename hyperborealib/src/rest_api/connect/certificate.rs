use serde_json::{json, Value as Json};

use crate::crypto::prelude::*;
use crate::rest_api::{AsJson, AsJsonError};

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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ConnectionCertificate {
    pub token: ConnectionToken,
    pub sign: Vec<u8>
}

impl ConnectionCertificate {
    pub fn new(client_secret: &SecretKey, server_public: PublicKey) -> Self {
        let token = ConnectionToken::new(server_public);

        let sign = client_secret.create_signature(token.to_bytes());

        Self {
            token,
            sign
        }
    }

    pub fn validate(&self, client_public: &PublicKey, server_public: &PublicKey) -> Result<bool, CryptographyError> {
        if &self.token.public_key != server_public {
            return Ok(false);
        }

        client_public.verify_signature(self.token.to_bytes(), &self.sign)
    }
}

impl AsJson for ConnectionCertificate {
    fn to_json(&self) -> Result<Json, AsJsonError> {
        Ok(json!({
            "token": base64_encode(self.token.to_bytes()),
            "sign": base64_encode(&self.sign)
        }))
    }

    fn from_json(json: &Json) -> Result<Self, AsJsonError> where Self: Sized {
        let Some(token) = json.get("token").and_then(Json::as_str) else {
            return Err(AsJsonError::FieldNotFound("token"));
        };

        let Some(sign) = json.get("sign").and_then(Json::as_str) else {
            return Err(AsJsonError::FieldNotFound("sign"));
        };

        Ok(Self {
            token: ConnectionToken::from_bytes(base64_decode(token)?)?,
            sign: base64_decode(sign)?
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize_token() -> Result<(), CryptographyError> {
        let public = SecretKey::random().public_key();

        let token = ConnectionToken::new(public);

        assert_eq!(ConnectionToken::from_bytes(token.to_bytes())?, token);

        Ok(())
    }

    #[test]
    fn serialize_certificate() -> Result<(), AsJsonError> {
        let secret = SecretKey::random();
        let public = SecretKey::random().public_key();

        let cert = ConnectionCertificate::new(&secret, public);

        assert_eq!(ConnectionCertificate::from_json(&cert.to_json()?)?, cert);

        Ok(())
    }
}
