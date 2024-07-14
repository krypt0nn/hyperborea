use serde_json::{json, Value as Json};

use crate::rest_api::prelude::{
    Client,
    Server,
    AsJson,
    AsJsonError
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Sender {
    pub client: Client,
    pub server: Server
}

impl Sender {
    #[inline]
    pub fn new(client: Client, server: Server) -> Self {
        Self {
            client,
            server
        }
    }
}

impl AsJson for Sender {
    fn to_json(&self) -> Result<Json, AsJsonError> {
        Ok(json!({
            "client": self.client.to_json()?,
            "server": self.server.to_json()?
        }))
    }

    fn from_json(json: &Json) -> Result<Self, AsJsonError> where Self: Sized {
        Ok(Self {
            client: json.get("client")
                .map(Client::from_json)
                .ok_or_else(|| AsJsonError::FieldNotFound("client"))??,

            server: json.get("server")
                .map(Server::from_json)
                .ok_or_else(|| AsJsonError::FieldNotFound("server"))??
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::crypto::asymmetric::SecretKey;
    use crate::rest_api::prelude::*;

    use super::*;

    #[test]
    fn serialize() -> Result<(), AsJsonError> {
        let client = SecretKey::random();
        let server = SecretKey::random();

        let info = ClientInfo::thin();
        let cert = ConnectionCertificate::new(&client, server.public_key());

        let client = Client::new(client.public_key(), cert, info);
        let server = Server::new(server.public_key(), "amogus");

        let sender = Sender::new(client, server);

        assert_eq!(Sender::from_json(&sender.to_json()?)?, sender);

        Ok(())
    }
}
