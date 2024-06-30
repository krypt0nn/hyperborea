use serde_json::Value as Json;

use crate::crypto::{
    SecretKey,
    PublicKey
};

use crate::rest_api::prelude::*;

mod client_info;
mod certificate;
mod request;
mod response;

pub use client_info::{
    ClientType,
    ClientInfo
};

pub use certificate::{
    ConnectionToken,
    ConnectionCertificate
};

pub use request::ConnectRequestBody;
pub use response::ConnectResponseBody;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ConnectRequest(pub Request<ConnectRequestBody>);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ConnectResponse(pub Response<ConnectResponseBody>);

impl ConnectRequest {
    #[inline]
    pub fn new(client_secret: &SecretKey, server_public: PublicKey, client: ClientInfo) -> Self {
        Self(Request::new(client_secret, ConnectRequestBody::new(client_secret, server_public, client)))
    }

    #[inline]
    pub fn validate(&self, server_public: &PublicKey) -> Result<bool, ValidationError> {
        Ok(self.0.validate()? && self.0.request.certificate.validate(&self.0.public_key, server_public)?)
    }
}

impl AsJson for ConnectRequest {
    #[inline]
    fn to_json(&self) -> Result<Json, AsJsonError> {
        self.0.to_json()
    }

    #[inline]
    fn from_json(json: &Json) -> Result<Self, AsJsonError> where Self: Sized {
        Ok(Self(Request::from_json(json)?))
    }
}

impl ConnectResponse {
    pub fn success(status: ResponseStatus, server_secret: &SecretKey, proof_seed: u64) -> Self {
        let proof = server_secret.create_signature(proof_seed.to_be_bytes());

        Self(Response::success(
            status,
            server_secret.public_key(),
            proof,
            ConnectResponseBody::new()
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

impl AsJson for ConnectResponse {
    #[inline]
    fn to_json(&self) -> Result<Json, AsJsonError> {
        self.0.to_json()
    }

    #[inline]
    fn from_json(json: &Json) -> Result<Self, AsJsonError> where Self: Sized {
        Ok(Self(Response::from_json(json)?))
    }
}
