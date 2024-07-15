use serde_json::{json, Value as Json};

use crate::crypto::prelude::*;
use crate::rest_api::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Client {
    pub public_key: PublicKey,
    pub certificate: ConnectionCertificate,
    pub info: ClientInfo
}

impl Client {
    #[inline]
    pub fn new(public_key: PublicKey, certificate: ConnectionCertificate, info: ClientInfo) -> Self {
        Self {
            public_key,
            certificate,
            info
        }
    }
}

impl AsJson for Client {
    fn to_json(&self) -> Result<Json, AsJsonError> {
        Ok(json!({
            "public_key": self.public_key.to_base64(),
            "certificate": self.certificate.to_json()?,
            "client": self.info.to_json()?
        }))
    }

    fn from_json(json: &Json) -> Result<Self, AsJsonError> where Self: Sized {
        let Some(public_key) = json.get("public_key").and_then(Json::as_str) else {
            return Err(AsJsonError::FieldNotFound("public_key"));
        };

        let Some(certificate) = json.get("certificate") else {
            return Err(AsJsonError::FieldNotFound("certificate"));
        };

        let Some(info) = json.get("client") else {
            return Err(AsJsonError::FieldNotFound("client"));
        };

        Ok(Client {
            public_key: PublicKey::from_base64(public_key)?,
            certificate: ConnectionCertificate::from_json(certificate)?,
            info: ClientInfo::from_json(info)?
        })
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::connection_certificate::tests::get_certificate;

    use super::*;

    pub fn get_client() -> Client {
        let public = SecretKey::random().public_key();

        let certificate = get_certificate();
        let info = ClientInfo::thin();

        Client::new(public, certificate, info)
    }

    #[test]
    fn serialize() -> Result<(), AsJsonError> {
        let client = get_client();

        assert_eq!(Client::from_json(&client.to_json()?)?, client);

        Ok(())
    }
}
