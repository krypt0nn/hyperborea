use serde_json::Value as Json;

use crate::crypto::prelude::*;
use crate::rest_api::prelude::*;

mod request;
mod response;

pub use request::*;
pub use response::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// `POST /api/v1/poll` request.
/// 
/// This request is used to poll a message (get and delete)
/// sent to the requesting client from the server's inbox.
/// 
/// Messaging API allows client to indirectly communicate with
/// each other without need of direct access to (and from) the internet.
pub struct PollRequest(pub Request<PollRequestBody>);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// `POST /api/v1/poll` response.
pub struct PollResponse(pub Response<PollResponseBody>);

impl PollRequest {
    #[inline]
    pub fn new(client_secret: &SecretKey, channel: impl ToString, limit: Option<u64>) -> Self {
        Self(Request::new(client_secret, PollRequestBody::new(channel, limit)))
    }

    #[inline]
    /// Validate the request.
    /// 
    /// Calls `validate()` function on the request's body.
    pub fn validate(&self) -> Result<bool, ValidationError> {
        self.0.validate()
    }
}

impl AsJson for PollRequest {
    #[inline]
    fn to_json(&self) -> Result<Json, AsJsonError> {
        self.0.to_json()
    }

    #[inline]
    fn from_json(json: &Json) -> Result<Self, AsJsonError> where Self: Sized {
        Ok(Self(Request::from_json(json)?))
    }
}

impl PollResponse {
    pub fn success(status: ResponseStatus, server_secret: &SecretKey, proof_seed: u64, response_body: PollResponseBody) -> Self {
        let proof = server_secret.create_signature(proof_seed.to_be_bytes());

        Self(Response::success(
            status,
            server_secret.public_key(),
            proof,
            response_body
        ))
    }

    #[inline]
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

impl AsJson for PollResponse {
    #[inline]
    fn to_json(&self) -> Result<Json, AsJsonError> {
        self.0.to_json()
    }

    #[inline]
    fn from_json(json: &Json) -> Result<Self, AsJsonError> where Self: Sized {
        Ok(Self(Response::from_json(json)?))
    }
}
