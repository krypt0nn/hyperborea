use serde_json::Value as Json;

use crate::crypto::prelude::*;
use crate::rest_api::prelude::*;

mod request;
mod response;

pub use request::LookupRequestBody;
pub use response::LookupResponseBody;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// `POST /api/v1/lookup` request.
/// 
/// This request is sent to the `POST /api/v1/lookup` to perform
/// client lookup in the chosen server's routing table.
pub struct LookupRequest(pub Request<LookupRequestBody>);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// `POST /api/v1/lookup` response.
pub struct LookupResponse(pub Response<LookupResponseBody>);

impl LookupRequest {
    #[inline]
    /// Craft new `POST /api/v1/lookup` response.
    /// 
    /// - `client_secret` must contain reference to the client's
    ///   secret key. It will be used to sign the request.
    /// 
    /// - `lookup_client_public` must contain public key
    ///   of the client we want to find in the server's
    ///   routing table.
    /// 
    /// - `lookup_client_type` should contain filter of the
    ///   client type we want to find, or `None` if any.
    ///   This is needed to split the public keys' namespace
    ///   for different client types.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use hyperborealib::crypto::prelude::*;
    /// use hyperborealib::rest_api::prelude::*;
    /// 
    /// // Secret key of the client who sends the lookup request
    /// let client_secret = SecretKey::random();
    /// 
    /// // Public key of the client we want to find
    /// let needed_client_public = SecretKey::random().public_key();
    /// 
    /// let request = LookupRequest::new(&client_secret, needed_client_public, None);
    /// ```
    pub fn new(client_secret: &SecretKey, lookup_client_public: PublicKey, lookup_client_type: Option<ClientType>) -> Self {
        Self(Request::new(client_secret, LookupRequestBody::new(lookup_client_public, lookup_client_type)))
    }

    #[inline]
    /// Validate the request.
    /// 
    /// Calls `validate()` function on the request's body.
    pub fn validate(&self) -> Result<bool, ValidationError> {
        self.0.validate()
    }
}

impl AsJson for LookupRequest {
    #[inline]
    fn to_json(&self) -> Result<Json, AsJsonError> {
        self.0.to_json()
    }

    #[inline]
    fn from_json(json: &Json) -> Result<Self, AsJsonError> where Self: Sized {
        Ok(Self(Request::from_json(json)?))
    }
}

impl LookupResponse {
    /// Create successful `POST /api/v1/lookup` response.
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
    /// - `response_body` must contain lookup response body.
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
    /// 
    /// let response = LookupResponse::success(
    ///     ResponseStatus::Success,
    ///     &SecretKey::random(),
    ///     safe_random_u64_long(), // Here must be the original request's proof seed
    ///     response_body
    /// );
    /// ```
    pub fn success(status: ResponseStatus, server_secret: &SecretKey, proof_seed: u64, response_body: LookupResponseBody) -> Self {
        let proof = server_secret.create_signature(proof_seed.to_be_bytes());

        Self(Response::success(
            status,
            server_secret.public_key(),
            proof,
            response_body
        ))
    }

    #[inline]
    /// Create failed `POST /api/v1/lookup` response.
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
    /// let response = LookupResponse::error(
    ///     ResponseStatus::ServerError,
    ///     "Example error"
    /// );
    /// ```
    pub fn error(status: ResponseStatus, reason: impl ToString) -> Self {
        Self(Response::error(status, reason))
    }

    #[inline]
    /// Validate the response.
    /// 
    /// Calls `validate()` function on the response's body.
    pub fn validate(&self, proof_seed: u64) -> Result<bool, ValidationError> {
        self.0.validate(proof_seed)
    }
}

impl AsJson for LookupResponse {
    #[inline]
    fn to_json(&self) -> Result<Json, AsJsonError> {
        self.0.to_json()
    }

    #[inline]
    fn from_json(json: &Json) -> Result<Self, AsJsonError> where Self: Sized {
        Ok(Self(Response::from_json(json)?))
    }
}
