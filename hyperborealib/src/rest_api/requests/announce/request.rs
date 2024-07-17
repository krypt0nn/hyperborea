use serde_json::{json, Value as Json};

use crate::rest_api::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[allow(clippy::large_enum_variant)]
/// `POST /api/v1/announce` request body.
/// 
/// Refer to the `AnnounceRequest` for details.
pub enum AnnounceRequestBody {
    Client {
        client: Client,
        server: Server
    },

    Server {
        server: Server
    }
}

impl AnnounceRequestBody {
    /// Create new `POST /api/v1/announce` client request body.
    /// 
    /// - `client` must contain information about the
    ///   announcing client.
    /// 
    /// - `server` must contain information about the
    ///   server to which this client is connected.
    pub fn client(client: Client, server: Server) -> Self {
        Self::Client {
            client,
            server
        }
    }

    #[inline]
    /// Create new `POST /api/v1/announce` server request body.
    /// 
    /// - `server` must contain information about the
    ///   server that is being announced.
    pub fn server(server: Server) -> Self {
        Self::Server {
            server
        }
    }
}

impl AsJson for AnnounceRequestBody {
    fn to_json(&self) -> Result<Json, AsJsonError> {
        match self {
            Self::Client { client, server } => {
                Ok(json!({
                    "announce": "client",
                    "client": client.to_json()?,
                    "server": server.to_json()?
                }))
            }

            Self::Server { server } => {
                Ok(json!({
                    "announce": "server",
                    "server": server.to_json()?,
                }))
            }
        }
    }

    fn from_json(json: &Json) -> Result<Self, AsJsonError> where Self: Sized {
        let Some(announce) = json.get("announce").and_then(Json::as_str) else {
            return Err(AsJsonError::FieldNotFound("announce"));
        };

        match announce {
            "client" => {
                Ok(Self::Client {
                    client: json.get("client")
                        .ok_or_else(|| AsJsonError::FieldNotFound("client"))
                        .and_then(Client::from_json)?,

                    server: json.get("server")
                        .ok_or_else(|| AsJsonError::FieldNotFound("server"))
                        .and_then(Server::from_json)?
                })
            }

            "server" => {
                Ok(Self::Server {
                    server: json.get("server")
                        .ok_or_else(|| AsJsonError::FieldNotFound("server"))
                        .and_then(Server::from_json)?
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
    fn serialize_client() -> Result<(), AsJsonError> {
        let client = get_client();
        let server = get_server();

        let request = AnnounceRequestBody::client(client, server);

        assert_eq!(AnnounceRequestBody::from_json(&request.to_json()?)?, request);

        Ok(())
    }

    #[test]
    fn serialize_server() -> Result<(), AsJsonError> {
        let server = get_server();

        let request = AnnounceRequestBody::server(server);

        assert_eq!(AnnounceRequestBody::from_json(&request.to_json()?)?, request);

        Ok(())
    }
}
