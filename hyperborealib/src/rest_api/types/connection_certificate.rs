use serde_json::{json, Value as Json};

use crate::crypto::prelude::*;

use crate::rest_api::{AsJson, AsJsonError};
use crate::rest_api::types::ConnectionToken;

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
pub(crate) mod tests {
    use super::*;

    pub fn get_certificate() -> ConnectionCertificate {
        let secret = SecretKey::random();
        let public = SecretKey::random().public_key();

        ConnectionCertificate::new(&secret, public)
    }

    #[test]
    fn serialize() -> Result<(), AsJsonError> {
        let cert = get_certificate();

        assert_eq!(ConnectionCertificate::from_json(&cert.to_json()?)?, cert);

        Ok(())
    }
}
