use serde_json::{json, Value as Json};

use crate::rest_api::{AsJson, AsJsonError};

use crate::rest_api::clients::Client as ClientApiRecord;
use crate::rest_api::servers::Server as ServerApiRecord;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LookupResponseBody {
    Local {
        client: ClientApiRecord,
        available: bool
    },

    Remote {
        client: ClientApiRecord,
        server: ServerApiRecord,
        available: bool
    },

    Hint {
        servers: Vec<ServerApiRecord>
    }
}

impl LookupResponseBody {
    #[inline]
    pub fn local(client: ClientApiRecord, available: bool) -> Self {
        Self::Local {
            client,
            available
        }
    }

    #[inline]
    pub fn remote(client: ClientApiRecord, server: ServerApiRecord, available: bool) -> Self {
        Self::Remote {
            client,
            server,
            available
        }
    }

    #[inline]
    pub fn hint(servers: Vec<ServerApiRecord>) -> Self {
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
                    client: ClientApiRecord::from_json(client)?,

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
                    client: ClientApiRecord::from_json(client)?,
                    server: ServerApiRecord::from_json(server)?,

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

mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[cfg(test)]
    use crate::rest_api::connect::{
        ConnectionCertificate,
        ClientInfo
    };

    #[test]
    fn serialize_local() -> Result<(), AsJsonError> {
        use crate::crypto::SecretKey;

        let secret = SecretKey::random();
        let public = SecretKey::random().public_key();

        let cert = ConnectionCertificate::new(&secret, public.clone());

        let client = ClientApiRecord::new(
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
        use crate::crypto::SecretKey;

        let secret = SecretKey::random();
        let public = SecretKey::random().public_key();

        let cert = ConnectionCertificate::new(&secret, public.clone());

        let client = ClientApiRecord::new(
            public.clone(),
            cert,
            ClientInfo::thin(),
        );

        let server = ServerApiRecord::new(public, "Hello, World!");

        let response = LookupResponseBody::remote(client, server, true);

        assert_eq!(LookupResponseBody::from_json(&response.to_json()?)?, response);

        Ok(())
    }

    #[test]
    fn serialize_hint() -> Result<(), AsJsonError> {
        use crate::crypto::SecretKey;

        let public = SecretKey::random().public_key();

        let response = LookupResponseBody::hint(vec![
            ServerApiRecord::new(public, "Hello, World!")
        ]);

        assert_eq!(LookupResponseBody::from_json(&response.to_json()?)?, response);

        Ok(())
    }
}
