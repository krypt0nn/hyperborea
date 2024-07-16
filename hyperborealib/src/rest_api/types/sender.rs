use serde_json::{json, Value as Json};

use crate::rest_api::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Description of the message's sender.
/// 
/// This is a standard type declared in the
/// hyperborea protocol's paper.
pub struct Sender {
    pub client: Client,
    pub server: Server
}

impl Sender {
    #[inline]
    /// Create description of the message sender.
    /// 
    /// - `client` must contain information about the client
    ///   which sent the message. This value is used to sign
    ///   the response message and address it to the server inbox.
    /// 
    /// - `server` must contain information about globally accessible
    ///   server which can be used by the client to receive the response
    ///   on the sent message. It is recommended to avoid using loopback
    ///   servers (the same value as in current server) because such things
    ///   can be blocked by the servers implementations.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use hyperborealib::crypto::prelude::*;
    /// use hyperborealib::rest_api::prelude::*;
    /// 
    /// let client = SecretKey::random();
    /// let server = SecretKey::random();
    /// 
    /// let certificate = ConnectionCertificate::new(&client, server.public_key());
    /// let info = ClientInfo::thin();
    /// 
    /// let client = Client::new(client.public_key(), certificate, info);
    /// let server = Server::new(server.public_key(), "example.org");
    /// 
    /// let sender = Sender::new(client, server);
    /// ```
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
pub(crate) mod tests {
    use super::client::tests::get_client;
    use super::server::tests::get_server;

    use super::*;

    pub fn get_sender() -> Sender {
        Sender::new(get_client(), get_server())
    }

    #[test]
    fn serialize() -> Result<(), AsJsonError> {
        let sender = get_sender();

        assert_eq!(Sender::from_json(&sender.to_json()?)?, sender);

        Ok(())
    }
}
