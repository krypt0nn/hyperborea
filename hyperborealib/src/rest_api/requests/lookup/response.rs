use serde_json::{json, Value as Json};

use crate::rest_api::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// `POST /api/v1/lookup` response body.
/// 
/// Refer to `LookupResponse` for details.
pub enum LookupResponseBody {
    /// Client is connected to the current server.
    Local {
        client: Client,
        available: bool
    },

    /// Client is connected to another server.
    Remote {
        client: Client,
        server: Server,
        available: bool
    },

    /// There's no info about the client. But there's
    /// a list of servers that could contain info about
    /// this client.
    /// 
    /// If you don't implement any special routing mechanism
    /// which supposed to give lookup hints - it's recommended
    /// to return list of all known servers here to reduce
    /// future possible network requests (`/api/v1/servers`).
    Hint {
        servers: Vec<Server>
    }
}

impl LookupResponseBody {
    #[inline]
    /// Craft `disposition: local` lookup response.
    /// 
    /// - `client` must contain information about the client.
    /// 
    /// - `available` must indicate whether the client is available.
    ///   This flag is managed by the server and doesn't have any
    ///   standard description.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use hyperborealib::crypto::prelude::*;
    /// use hyperborealib::rest_api::prelude::*;
    /// 
    /// let client_secret = SecretKey::random();
    /// let server_secret = SecretKey::random();
    /// 
    /// let client_public = client_secret.public_key();
    /// let server_public = server_secret.public_key();
    /// 
    /// let certificate = ConnectionCertificate::new(&client_secret, server_public);
    /// 
    /// let client_info = ClientInfo::thin();
    /// 
    /// let client = Client::new(client_public, certificate, client_info);
    /// 
    /// let response_body = LookupResponseBody::local(client, true);
    /// ```
    pub fn local(client: Client, available: bool) -> Self {
        Self::Local {
            client,
            available
        }
    }

    #[inline]
    /// Craft `disposition: remote` lookup response.
    /// 
    /// - `client` must contain information about the client.
    /// 
    /// - `server` must contain information about the server to which
    ///   this `client` is connected.
    /// 
    /// - `available` must indicate whether the client is available.
    ///   This flag is managed by the server and doesn't have any
    ///   standard description.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use hyperborealib::crypto::prelude::*;
    /// use hyperborealib::rest_api::prelude::*;
    /// 
    /// let client_secret = SecretKey::random();
    /// let server_secret = SecretKey::random();
    /// 
    /// let client_public = client_secret.public_key();
    /// let server_public = server_secret.public_key();
    /// 
    /// let certificate = ConnectionCertificate::new(&client_secret, server_public.clone());
    /// 
    /// let client_info = ClientInfo::thin();
    /// 
    /// let client = Client::new(client_public, certificate, client_info);
    /// let server = Server::new(server_public, "example.org");
    /// 
    /// let response_body = LookupResponseBody::remote(client, server, true);
    /// ```
    pub fn remote(client: Client, server: Server, available: bool) -> Self {
        Self::Remote {
            client,
            server,
            available
        }
    }

    #[inline]
    /// Craft `disposition: hint` lookup response.
    /// 
    /// - `servers` should contain list of servers which
    ///   can know the needed client.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use hyperborealib::crypto::prelude::*;
    /// use hyperborealib::rest_api::prelude::*;
    /// 
    /// let response_body = LookupResponseBody::hint(vec![
    ///     Server::new(SecretKey::random().public_key(), "example1.org"),
    ///     Server::new(SecretKey::random().public_key(), "example2.org"),
    ///     Server::new(SecretKey::random().public_key(), "example3.org")
    /// ]);
    /// ```
    pub fn hint(servers: impl Into<Vec<Server>>) -> Self {
        Self::Hint {
            servers: servers.into()
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
    use crate::rest_api::types::client::tests::get_client;
    use crate::rest_api::types::server::tests::get_server;

    use super::*;

    #[test]
    fn serialize_local() -> Result<(), AsJsonError> {
        let client = get_client();

        let response = LookupResponseBody::local(client, true);

        assert_eq!(LookupResponseBody::from_json(&response.to_json()?)?, response);

        Ok(())
    }

    #[test]
    fn serialize_remote() -> Result<(), AsJsonError> {
        let client = get_client();
        let server = get_server();

        let response = LookupResponseBody::remote(client, server, true);

        assert_eq!(LookupResponseBody::from_json(&response.to_json()?)?, response);

        Ok(())
    }

    #[test]
    fn serialize_hint() -> Result<(), AsJsonError> {
        let response = LookupResponseBody::hint(vec![
            get_server(),
            get_server(),
            get_server()
        ]);

        assert_eq!(LookupResponseBody::from_json(&response.to_json()?)?, response);

        Ok(())
    }
}
