use serde_json::Value as Json;

use crate::rest_api::{AsJson, AsJsonError};

use crate::rest_api::connect::ClientType;

use crate::crypto::{
    SecretKey,
    PublicKey
};

use super::request::Request;
use super::response::Response;

use super::ValidationError;
use super::status::ResponseStatus;

mod request;
mod response;

pub use request::LookupRequestBody;
pub use response::LookupResponseBody;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LookupRequest(pub Request<LookupRequestBody>);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LookupResponse(pub Response<LookupResponseBody>);

impl LookupRequest {
    #[inline]
    pub fn new(local_client_secret: &SecretKey, lookup_client_public: PublicKey, lookup_client_type: Option<ClientType>) -> Self {
        Self(Request::new(local_client_secret, request::LookupRequestBody::new(lookup_client_public, lookup_client_type)))
    }

    #[inline]
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
    pub fn success(status: ResponseStatus, server_secret: &SecretKey, proof_seed: u64, response_body: LookupResponseBody) -> Self {
        let proof = server_secret.create_signature(proof_seed.to_be_bytes());

        Self(Response::success(
            status,
            server_secret.public_key(),
            proof,
            response_body
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
