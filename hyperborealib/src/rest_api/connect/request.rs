use serde_json::{json, Value as Json};

use crate::rest_api::{AsJson, AsJsonError};

use crate::crypto::{
    SecretKey,
    PublicKey
};

use super::certificate::ConnectionCertificate;
use super::client_info::ClientInfo;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ConnectRequestBody {
    pub certificate: ConnectionCertificate,
    pub client: ClientInfo
}

impl ConnectRequestBody {
    pub fn new(client_secret: &SecretKey, server_public: PublicKey, client: ClientInfo) -> Self {
        Self {
            certificate: ConnectionCertificate::new(client_secret, server_public),
            client
        }
    }
}

impl AsJson for ConnectRequestBody {
    fn to_json(&self) -> Result<Json, AsJsonError> {
        Ok(json!({
            "certificate": self.certificate.to_json()?,
            "client": self.client.to_json()?
        }))
    }

    fn from_json(json: &Json) -> Result<Self, AsJsonError> where Self: Sized {
        let Some(certificate) = json.get("certificate") else {
            return Err(AsJsonError::FieldNotFound("certificate"));
        };

        let Some(client) = json.get("client") else {
            return Err(AsJsonError::FieldNotFound("client"));
        };

        Ok(Self {
            certificate: ConnectionCertificate::from_json(certificate)?,
            client: ClientInfo::from_json(client)?
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize() -> Result<(), AsJsonError> {
        let secret = SecretKey::random();
        let public = SecretKey::random().public_key();

        let request = ConnectRequestBody::new(&secret, public, ClientInfo::thin());

        assert_eq!(ConnectRequestBody::from_json(&request.to_json()?)?, request);

        Ok(())
    }
}
