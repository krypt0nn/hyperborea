use serde_json::Value as Json;

use crate::crypto::prelude::*;
use crate::rest_api::prelude::*;

mod request;
mod response;

pub use request::SendRequestBody;
pub use response::SendResponseBody;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SendRequest(pub Request<SendRequestBody>);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SendResponse(pub Response<SendResponseBody>);

impl SendRequest {
    #[inline]
    pub fn new(client_secret: &SecretKey, sender: Sender, receiver_public: PublicKey, channel: impl ToString, message: Message) -> Self {
        Self(Request::new(client_secret, SendRequestBody::new(sender, receiver_public, channel, message)))
    }

    #[inline]
    pub fn validate(&self) -> Result<bool, ValidationError> {
        self.0.validate()
    }
}

impl AsJson for SendRequest {
    #[inline]
    fn to_json(&self) -> Result<Json, AsJsonError> {
        self.0.to_json()
    }

    #[inline]
    fn from_json(json: &Json) -> Result<Self, AsJsonError> where Self: Sized {
        Ok(Self(Request::from_json(json)?))
    }
}

impl SendResponse {
    pub fn success(status: ResponseStatus, server_secret: &SecretKey, proof_seed: u64) -> Self {
        let proof = server_secret.create_signature(proof_seed.to_be_bytes());

        Self(Response::success(
            status,
            server_secret.public_key(),
            proof,
            SendResponseBody::new()
        ))
    }

    pub fn error(status: ResponseStatus, reason: impl ToString) -> Self {
        Self(Response::error(status, reason))
    }

    #[inline]
    pub fn validate(&self, proof_seed: u64) -> Result<bool, ValidationError> {
        self.0.validate(proof_seed)
    }
}

impl AsJson for SendResponse {
    #[inline]
    fn to_json(&self) -> Result<Json, AsJsonError> {
        self.0.to_json()
    }

    #[inline]
    fn from_json(json: &Json) -> Result<Self, AsJsonError> where Self: Sized {
        Ok(Self(Response::from_json(json)?))
    }
}
