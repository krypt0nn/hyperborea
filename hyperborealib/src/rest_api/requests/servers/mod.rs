use serde_json::{json, Value as Json};

use crate::rest_api::prelude::*;

use crate::STANDARD_VERSION;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ServersResponse {
    pub standard: u64,
    pub servers: Vec<Server>
}

impl ServersResponse {
    pub fn new(servers: impl Into<Vec<Server>>) -> Self {
        Self {
            standard: STANDARD_VERSION,
            servers: servers.into()
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
