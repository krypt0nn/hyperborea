use serde_json::{json, Value as Json};

use crate::rest_api::prelude::*;

use crate::STANDARD_VERSION;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// `GET /api/v1/servers` response.
/// 
/// This response should contain list of servers
/// known by the current server. This list can be used
/// by other clients and servers to construct the map
/// of the network and fill their own routing tables.
/// 
/// By providing useful information here you reduce
/// total amount of requests sent within the network
/// to lookup the clients.
pub struct ServersResponse {
    pub standard: u64,
    pub servers: Vec<Server>
}

impl ServersResponse {
    /// Create new `GET /api/v1/servers` response.
    /// 
    /// - `servers` should contain list of all
    ///   known servers.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use hyperborealib::crypto::prelude::*;
    /// use hyperborealib::rest_api::prelude::*;
    /// 
    /// let response = ServersResponse::new(vec![
    ///     Server::new(SecretKey::random().public_key(), "example1.org"),
    ///     Server::new(SecretKey::random().public_key(), "example2.org"),
    ///     Server::new(SecretKey::random().public_key(), "example3.org")
    /// ]);
    /// ```
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
    use crate::rest_api::types::server::tests::get_server;

    use super::*;

    #[test]
    fn serialize() -> Result<(), AsJsonError> {
        let response = ServersResponse::new(vec![
            get_server(),
            get_server(),
            get_server()
        ]);

        assert_eq!(ServersResponse::from_json(&response.to_json()?)?, response);

        Ok(())
    }
}
