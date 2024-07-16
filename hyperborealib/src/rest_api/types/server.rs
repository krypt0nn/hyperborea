use serde_json::{json, Value as Json};

use crate::crypto::prelude::*;
use crate::rest_api::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Description of the remote server.
/// 
/// This is a standard type declared in the
/// hyperborea protocol's paper.
pub struct Server {
    pub public_key: PublicKey,
    pub address: String
}

impl Server {
    #[inline]
    /// Create server description.
    /// 
    /// - `public_key` must contain public key of the server.
    ///   Later it is used for digital signatures signing and
    ///   creation of the shared secret encryption keys.
    /// 
    /// - `address` must contain globally available address of
    ///   this server (either IP or domain name). It may contain
    ///   any arbitrary data, but then this server will not be seen
    ///   by other network participants.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use hyperborealib::crypto::prelude::*;
    /// use hyperborealib::rest_api::prelude::*;
    /// 
    /// let server_public = SecretKey::random().public_key();
    /// 
    /// let server = Server::new(server_public, "example.org");
    /// ```
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

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    pub fn get_server() -> Server {
        Server::new(SecretKey::random().public_key(), "localhost:8001")
    }

    #[test]
    fn serialize() -> Result<(), AsJsonError> {
        let server = get_server();

        assert_eq!(Server::from_json(&server.to_json()?)?, server);

        Ok(())
    }
}
