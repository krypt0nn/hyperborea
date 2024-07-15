use serde_json::{json, Value as Json};

use crate::crypto::prelude::*;
use crate::rest_api::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// `POST /api/v1/connect` request body.
/// 
/// This request is sent to the `POST /api/v1/connect` to
/// perform client connection to the chosen server. Connected
/// clients are linked to their servers by the connection
/// certificates. They should be used to identify to which
/// server a client is connected if there's two or more records
/// of this client connected to different servers. In this case
/// one with newest certificate is chosen.
pub struct ConnectRequestBody {
    pub certificate: ConnectionCertificate,
    pub client: ClientInfo
}

impl ConnectRequestBody {
    /// Create connect request body from given ingredients.
    /// 
    /// - `client_secret` must contain reference to the connecting client's
    ///   secret key. It is used to sign the connection certificate.
    /// 
    /// - `server_public` must contain the public key of the server to which
    ///   this client is being connected. It is used in the connection certificate.
    /// 
    /// - `client` must contain information about connecting client.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use hyperborealib::rest_api::prelude::*;
    /// use hyperborealib::crypto::prelude::*;
    /// 
    /// let client_secret = SecretKey::random();
    /// let server_public = SecretKey::random().public_key();
    /// 
    /// let client = ClientInfo::thin();
    /// 
    /// let request_body = ConnectRequestBody::new(&client_secret, server_public, client);
    /// ```
    pub fn new(client_secret: &SecretKey, server_public: PublicKey, client: ClientInfo) -> Self {
        Self {
            certificate: ConnectionCertificate::new(client_secret, server_public),
            client
        }
    }

    #[inline]
    /// Create connect request body with pre-defined values.
    /// 
    /// - `client` must contain information about the current
    ///   client that wants to connect to the server.
    /// 
    /// - `certificate` must contain connection certificate of
    ///   this client signed for the destination server.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use hyperborealib::rest_api::prelude::*;
    /// use hyperborealib::crypto::prelude::*;
    /// 
    /// // Secret key of the client that is used to sign the certificate
    /// let client_secret = SecretKey::random();
    /// 
    /// // Public key of the destination server that is used to sign
    /// // the certificate and prove that the client is connected
    /// // to this server
    /// let server_public = SecretKey::random().public_key();
    /// 
    /// // Prepare information about the client
    /// let client = ClientInfo::thin();
    /// let certificate = ConnectionCertificate::new(&client_secret, server_public);
    /// 
    /// // Craft the request body
    /// let request_body = ConnectRequestBody::from_certificate(client, certificate);
    /// ```
    pub fn from_certificate(client: ClientInfo, certificate: ConnectionCertificate) -> Self {
        Self {
            client,
            certificate
        }
    }
}

impl AsJson for ConnectRequestBody {
    fn to_json(&self) -> Result<Json, AsJsonError> {
        Ok(json!({
            "certificate": self.certificate.to_json()?,
            "client": self.client.to_json()?
        }))
    }

    fn from_json(json: &Json) -> Result<Self, AsJsonError> where Self: Sized {
        let Some(certificate) = json.get("certificate") else {
            return Err(AsJsonError::FieldNotFound("certificate"));
        };

        let Some(client) = json.get("client") else {
            return Err(AsJsonError::FieldNotFound("client"));
        };

        Ok(Self {
            certificate: ConnectionCertificate::from_json(certificate)?,
            client: ClientInfo::from_json(client)?
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize() -> Result<(), AsJsonError> {
        let secret = SecretKey::random();
        let public = SecretKey::random().public_key();

        let request = ConnectRequestBody::new(&secret, public, ClientInfo::thin());

        assert_eq!(ConnectRequestBody::from_json(&request.to_json()?)?, request);

        Ok(())
    }
}
