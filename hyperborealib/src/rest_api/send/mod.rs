use serde_json::Value as Json;

use crate::rest_api::prelude::*;

use crate::crypto::{
    PublicKey,
    SecretKey,
    Error as CryptographyError
};

mod text_encoding;
mod text_encryption;
mod text_compression;
mod message_encoding;
mod message;
mod sender;

mod request;
mod response;

pub use text_encoding::*;
pub use text_encryption::*;
pub use text_compression::*;
pub use message_encoding::*;
pub use message::*;
pub use sender::*;

pub use request::SendRequestBody;
pub use response::SendResponseBody;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Wrong message encoding format provided: '{0}'")]
    WrongMessageEncodingFormat(String),

    #[error("Message's signature is invalid")]
    InvalidMessageSignature,

    #[error("Failed to decode text: {0}")]
    TextDecodingError(#[source] Box<dyn std::error::Error>),

    #[error("Failed to encrypt text: {0}")]
    TextEncryptionFailed(#[source] Box<dyn std::error::Error>),

    #[error("Failed to decrypt text: {0}")]
    TextDecryptionFailed(#[source] Box<dyn std::error::Error>),

    #[error("Failed to compress text: {0}")]
    TextCompressionFailed(#[source] Box<dyn std::error::Error>),

    #[error("Failed to decompress text: {0}")]
    TextDecompressionFailed(#[source] Box<dyn std::error::Error>),

    #[error(transparent)]
    CryptographyError(#[from] CryptographyError)
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SendRequest(pub Request<SendRequestBody>);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
