use serde_json::{json, Value as Json};

use crate::rest_api::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum LookupResponseBody {
    Local {
        client: Client,
        available: bool
    },

    Remote {
        client: Client,
        server: Server,
        available: bool
    },

    Hint {
        servers: Vec<Server>
    }
}

impl LookupResponseBody {
    #[inline]
    pub fn local(client: Client, available: bool) -> Self {
        Self::Local {
            client,
            available
        }
    }

    #[inline]
    pub fn remote(client: Client, server: Server, available: bool) -> Self {
        Self::Remote {
            client,
            server,
            available
        }
    }

    #[inline]
    pub fn hint(servers: Vec<Server>) -> Self {
        Self::Hint {
            servers
        }
    }
}

impl AsJson for LookupResponseBody {
    fn to_json(&self) -> Result<Json, AsJsonError> {
        match self {
            Self::Local { client, available } => {
                Ok(json!({
                    "disposition": "local",
                    "result": {
                        "client": client.to_json()?,
                        "available": available
                    }
                }))
            }

            Self::Remote { client, server, available } => {
                Ok(json!({
                    "disposition": "remote",
                    "result": {
                        "client": client.to_json()?,
                        "server": server.to_json()?,
                        "available": available
                    }
                }))
            }

            Self::Hint { servers } => {
                Ok(json!({
                    "disposition": "hint",
                    "result": {
                        "servers": servers.iter()
                            .map(AsJson::to_json)
                            .collect::<Result<Vec<_>, _>>()?
                    }
                }))
            }
        }
    }

    fn from_json(json: &Json) -> Result<Self, AsJsonError> where Self: Sized {
        let Some(disposition) = json.get("disposition").and_then(Json::as_str) else {
            return Err(AsJsonError::FieldNotFound("disposition"));
        };

        let Some(result) = json.get("result") else {
            return Err(AsJsonError::FieldNotFound("result"));
        };

        match disposition {
            "local" => {
                let Some(client) = result.get("client") else {
                    return Err(AsJsonError::FieldNotFound("result.client"));
                };

                Ok(Self::Local {
                    client: Client::from_json(client)?,

                    available: result.get("available")
                        .and_then(Json::as_bool)
                        .ok_or_else(|| AsJsonError::FieldNotFound("result.available"))?
                })
            }

            "remote" => {
                let Some(client) = result.get("client") else {
                    return Err(AsJsonError::FieldNotFound("result.client"));
                };

                let Some(server) = result.get("server") else {
                    return Err(AsJsonError::FieldNotFound("result.server"));
                };

                Ok(Self::Remote {
                    client: Client::from_json(client)?,
                    server: Server::from_json(server)?,

                    available: result.get("available")
                        .and_then(Json::as_bool)
                        .ok_or_else(|| AsJsonError::FieldNotFound("result.available"))?
                })
            }

            "hint" => {
                Ok(Self::Hint {
                    servers: result.get("servers")
                        .and_then(Json::as_array)
                        .ok_or_else(|| AsJsonError::FieldNotFound("result.servers"))?
                        .iter()
                        .map(AsJson::from_json)
                        .collect::<Result<Vec<_>, _>>()?
                })
            }

            _ => Err(AsJsonError::FieldValueInvalid("Field 'disposition' contains invalid format"))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::crypto::asymmetric::SecretKey;

    use super::*;

    #[test]
    fn serialize_local() -> Result<(), AsJsonError> {
        let secret = SecretKey::random();
        let public = SecretKey::random().public_key();

        let cert = ConnectionCertificate::new(&secret, public.clone());

        let client = Client::new(
            public,
            cert,
            ClientInfo::thin(),
        );

        let response = LookupResponseBody::local(client, true);

        assert_eq!(LookupResponseBody::from_json(&response.to_json()?)?, response);

        Ok(())
    }

    #[test]
    fn serialize_remote() -> Result<(), AsJsonError> {
        let secret = SecretKey::random();
        let public = SecretKey::random().public_key();

        let cert = ConnectionCertificate::new(&secret, public.clone());

        let client = Client::new(
            public.clone(),
            cert,
            ClientInfo::thin(),
        );

        let server = Server::new(public, "Hello, World!");

        let response = LookupResponseBody::remote(client, server, true);

        assert_eq!(LookupResponseBody::from_json(&response.to_json()?)?, response);

        Ok(())
    }

    #[test]
    fn serialize_hint() -> Result<(), AsJsonError> {
        let public = SecretKey::random().public_key();

        let response = LookupResponseBody::hint(vec![
            Server::new(public, "Hello, World!")
        ]);

        assert_eq!(LookupResponseBody::from_json(&response.to_json()?)?, response);

        Ok(())
    }
}
