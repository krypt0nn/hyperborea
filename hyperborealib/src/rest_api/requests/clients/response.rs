use serde_json::{json, Value as Json};

use crate::rest_api::prelude::*;

use crate::STANDARD_VERSION;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ClientsResponse {
    pub standard: u64,
    pub clients: Vec<Client>
}

impl ClientsResponse {
    /// Create new clients request response.
    /// 
    /// This response is sent after the `GET /api/v1/clients` request.
    /// It should contain list of all the clients connected to the current server.
    /// This request can be used by other servers or clients to fulfill their own
    /// routing table or to perform custom client lookups.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use hyperborealib::crypto::prelude::*;
    /// use hyperborealib::rest_api::prelude::*;
    /// 
    /// // Public and secret keys of the client
    /// let secret_key = SecretKey::random();
    /// let public_key = secret_key.public_key();
    /// 
    /// // Public key of some random server
    /// let server_public = SecretKey::random().public_key();
    /// 
    /// // Connection certificate of this client to the random server
    /// let certificate = ConnectionCertificate::new(secret_key, server_public);
    /// 
    /// // Information about the client
    /// let info = ClientInfo::thin();
    /// 
    /// // Client description
    /// let client = Client::new(public_key, certificate, info);
    /// 
    /// // Clients request response
    /// let response = ClientsResponse::new(vec![
    ///     client
    /// ]);
    /// ```
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
    use crate::rest_api::types::client::tests::get_client;

    use super::*;

    #[test]
    fn serialize() -> Result<(), AsJsonError> {
        let response = ClientsResponse::new(vec![
            get_client(),
            get_client(),
            get_client()
        ]);

        assert_eq!(ClientsResponse::from_json(&response.to_json()?)?, response);

        Ok(())
    }
}
