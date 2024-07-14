use serde_json::{json, Value as Json};

use crate::crypto::asymmetric::PublicKey;

use crate::rest_api::{AsJson, AsJsonError};

use crate::STANDARD_VERSION;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Server {
    pub public_key: PublicKey,
    pub address: String
}

impl Server {
    #[inline]
    pub fn new(public_key: PublicKey, address: impl ToString) -> Self {
        Self {
            public_key,
            address: address.to_string()
        }
    }
}

impl AsJson for Server {
    fn to_json(&self) -> Result<Json, AsJsonError> {
        Ok(json!({
            "public_key": self.public_key.to_base64(),
            "address": self.address
        }))
    }

    fn from_json(json: &Json) -> Result<Self, AsJsonError> where Self: Sized {
        let Some(public_key) = json.get("public_key").and_then(Json::as_str) else {
            return Err(AsJsonError::FieldNotFound("public_key"));
        };

        let Some(address) = json.get("address").and_then(Json::as_str) else {
            return Err(AsJsonError::FieldNotFound("address"));
        };

        Ok(Server {
            public_key: PublicKey::from_base64(public_key)?,
            address: address.to_string()
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ServersResponse {
    pub standard: u64,
    pub servers: Vec<Server>
}

impl ServersResponse {
    pub fn new(servers: Vec<Server>) -> Self {
        Self {
            standard: STANDARD_VERSION,
            servers
        }
    }
}

impl AsJson for ServersResponse {
    fn to_json(&self) -> Result<Json, AsJsonError> {
        match self.standard {
            1 => Ok(json!({
                "standard": self.standard,
                "servers": self.servers.iter()
                    .map(AsJson::to_json)
                    .collect::<Result<Vec<_>, AsJsonError>>()?
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
                let Some(servers) = json.get("servers").and_then(Json::as_array) else {
                    return Err(AsJsonError::FieldNotFound("servers"));
                };

                Ok(Self {
                    standard,
                    servers: servers.iter()
                        .map(AsJson::from_json)
                        .collect::<Result<Vec<_>, AsJsonError>>()?
                })
            }

            _ => Err(AsJsonError::InvalidStandard(standard))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::crypto::asymmetric::SecretKey;

    use super::*;

    #[test]
    fn serialize_server() -> Result<(), AsJsonError> {
        let server = Server {
            public_key: SecretKey::random().public_key(),
            address: String::from("Hello, World!")
        };

        assert_eq!(Server::from_json(&server.to_json()?)?, server);

        Ok(())
    }

    #[test]
    fn serialize_response() -> Result<(), AsJsonError> {
        let response = ServersResponse::new(vec![
            Server {
                public_key: SecretKey::random().public_key(),
                address: String::from("Hello, World!")
            }
        ]);

        assert_eq!(ServersResponse::from_json(&response.to_json()?)?, response);

        Ok(())
    }
}
