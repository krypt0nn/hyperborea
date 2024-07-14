use serde_json::{json, Value as Json};

use crate::crypto::asymmetric::PublicKey;

use crate::rest_api::prelude::*;

use crate::STANDARD_VERSION;

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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ClientsResponse {
    pub standard: u64,
    pub clients: Vec<Client>
}

impl ClientsResponse {
    pub fn new(clients: Vec<Client>) -> Self {
        Self {
            standard: STANDARD_VERSION,
            clients
        }
    }
}

impl AsJson for ClientsResponse {
    fn to_json(&self) -> Result<Json, AsJsonError> {
        match self.standard {
            1 => Ok(json!({
                "standard": self.standard,
                "clients": self.clients.iter()
                    .map(AsJson::to_json)
                    .collect::<Result<Vec<_>, _>>()?
            })),

            _ => Err(AsJsonError::InvalidStandard(self.standard))
        }
    }

    fn from_json(json: &Json) -> Result<Self, AsJsonError> where Self: Sized {
        let Some(standard) = json.get("standard").and_then(Json::as_u64) else {
            return Err(AsJsonError::FieldNotFound("standard"));
        };

        match standard {
            1 => {
                let Some(clients) = json.get("clients").and_then(Json::as_array) else {
                    return Err(AsJsonError::FieldNotFound("clients"));
                };

                Ok(Self {
                    standard,
                    clients: clients.iter()
                        .map(AsJson::from_json)
                        .collect::<Result<Vec<_>, _>>()?
                })
            }

            _ => Err(AsJsonError::InvalidStandard(standard))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::crypto::asymmetric::SecretKey;
    use crate::rest_api::connect::ConnectionCertificate;

    use super::*;

    #[test]
    fn serialize_client() -> Result<(), AsJsonError> {
        let secret = SecretKey::random();
        let public = SecretKey::random().public_key();

        let cert = ConnectionCertificate::new(
            &secret,
            public.clone()
        );

        let client = Client {
            public_key: public,
            certificate: cert,
            info: ClientInfo::thin()
        };

        assert_eq!(Client::from_json(&client.to_json()?)?, client);

        Ok(())
    }

    #[test]
    fn serialize_response() -> Result<(), AsJsonError> {
        let secret = SecretKey::random();
        let public = SecretKey::random().public_key();

        let cert = ConnectionCertificate::new(
            &secret,
            public.clone()
        );

        let response = ClientsResponse::new(vec![
            Client {
                public_key: public,
                certificate: cert,
                info: ClientInfo::thin()
            }
        ]);

        assert_eq!(ClientsResponse::from_json(&response.to_json()?)?, response);

        Ok(())
    }
}
