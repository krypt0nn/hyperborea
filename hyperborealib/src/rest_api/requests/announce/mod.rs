use serde_json::Value as Json;

use crate::crypto::prelude::*;
use crate::rest_api::prelude::*;

mod request;
mod response;

pub use request::AnnounceRequestBody;
pub use response::AnnounceResponseBody;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// `POST /api/v1/announce` request.
/// 
/// This request is sent to the `POST /api/v1/connect` to
/// perform client connection to the chosen server. Connected
/// clients are linked to their servers by the connection
/// certificates. They should be used to identify to which
/// server a client is connected if there's two or more records
/// of this client connected to different servers. In this case
/// one with newest certificate is chosen.
pub struct AnnounceRequest(pub Request<AnnounceRequestBody>);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// `POST /api/v1/announce` response.
pub struct AnnounceResponse(pub Response<AnnounceResponseBody>);

impl AnnounceRequest {
    #[inline]
    /// Craft new `POST /api/v1/announce` client request.
    /// 
    /// - `client_secret` must contain reference to the
    ///   client's secret key. It is used to sign the proof
    ///   and connection certificate to the server.
    /// 
    /// - `client` must contain information about the announcing client.
    /// 
    /// - `server` must contain information about the server to which
    ///   the `client` is connected.
    pub fn client(client_secret: &SecretKey, client: Client, server: Server) -> Self {
        Self(Request::new(client_secret, AnnounceRequestBody::client(client, server)))
    }

    #[inline]
    /// Craft new `POST /api/v1/announce` server request.
    /// 
    /// - `client_secret` must contain reference to the
    ///   client's secret key. It is used to sign the proof
    ///   and connection certificate to the server.
    /// 
    /// - `server` must contain information about the announcing server.
    pub fn server(client_secret: &SecretKey, server: Server) -> Self {
        Self(Request::new(client_secret, AnnounceRequestBody::server(server)))
    }

    /// Validate the request.
    /// 
    /// Calls `validate()` function on the request's body
    /// and verifies that the provided connection certificate
    /// is signed for the specified server.
    /// 
    /// - `server_public` must contain reference to the
    ///   public key of the current server to which
    ///   connection certificate was supposed to be signed.
    pub fn validate(&self) -> Result<bool, ValidationError> {
        let mut valid_cert = true;

        // Validate that the client is connected to the server.
        if let AnnounceRequestBody::Client { client, server } = &self.0.request {
            valid_cert = client.certificate.validate(&client.public_key, &server.public_key)?;
        }

        Ok(valid_cert && self.0.validate()?)
    }
}

impl AsJson for AnnounceRequest {
    #[inline]
    fn to_json(&self) -> Result<Json, AsJsonError> {
        self.0.to_json()
    }

    #[inline]
    fn from_json(json: &Json) -> Result<Self, AsJsonError> where Self: Sized {
        Ok(Self(Request::from_json(json)?))
    }
}

impl AnnounceResponse {
    /// Create successful `POST /api/v1/announce` response.
    /// 
    /// - `status` must contain status code of the response
    ///   (`100 Success` in most cases).
    /// 
    /// - `server_secret` must contain reference to the
    ///   secret key of the responding server. It is used
    ///   to sign the response's proof.
    /// 
    /// - `proof_seed` must contain the same seed as used
    ///   in the original request.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use hyperborealib::crypto::prelude::*;
    /// use hyperborealib::rest_api::prelude::*;
    /// 
    /// let response = AnnounceResponse::success(
    ///     ResponseStatus::Success,
    ///     &SecretKey::random(),
    ///     safe_random_u64_long() // Here must be the original request's proof seed
    /// );
    /// ```
    pub fn success(status: ResponseStatus, server_secret: &SecretKey, proof_seed: u64) -> Self {
        let proof = server_secret.create_signature(proof_seed.to_be_bytes());

        Self(Response::success(
            status,
            server_secret.public_key(),
            proof,
            AnnounceResponseBody::new()
        ))
    }

    #[inline]
    /// Create failed `POST /api/v1/announce` response.
    /// 
    /// - `status` must contain response's status.
    /// 
    /// - `reason` must contain error reason (message and/or description).
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use hyperborealib::crypto::prelude::*;
    /// use hyperborealib::rest_api::prelude::*;
    /// 
    /// let response = AnnounceResponse::error(
    ///     ResponseStatus::ServerError,
    ///     "Example error"
    /// );
    /// ```
    pub fn error(status: ResponseStatus, reason: impl ToString) -> Self {
        Self(Response::error(status, reason))
    }

    #[inline]
    /// Validate the request.
    /// 
    /// Calls `validate()` function on the response's body.
    pub fn validate(&self, proof_seed: u64) -> Result<bool, ValidationError> {
        self.0.validate(proof_seed)
    }
}

impl AsJson for AnnounceResponse {
    #[inline]
    fn to_json(&self) -> Result<Json, AsJsonError> {
        self.0.to_json()
    }

    #[inline]
    fn from_json(json: &Json) -> Result<Self, AsJsonError> where Self: Sized {
        Ok(Self(Response::from_json(json)?))
    }
}
