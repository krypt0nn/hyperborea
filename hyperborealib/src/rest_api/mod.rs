use serde_json::Value as Json;

use crate::crypto::Error as CryptographyError;

pub mod request;
pub mod response;
pub mod status;
pub mod middleware;

pub mod info;
pub mod clients;
pub mod servers;
pub mod connect;
pub mod lookup;
pub mod send;
pub mod poll;

#[derive(Debug, thiserror::Error)]
pub enum AsJsonError {
    #[error("Invalid standard version: {0}")]
    InvalidStandard(u64),

    #[error("Field `{0}` is not specified")]
    FieldNotFound(&'static str),

    #[error("Field `{0}` has invalid value")]
    FieldValueInvalid(&'static str),

    #[error(transparent)]
    Base64Error(#[from] base64::DecodeError),

    #[error(transparent)]
    CryptographyError(#[from] CryptographyError),

    #[error(transparent)]
    Other(#[from] Box<dyn std::error::Error>)
}

#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Proof seed must be a 64 bit long unsigned integer")]
    InvalidSeed,

    #[error(transparent)]
    CryptographyError(#[from] CryptographyError)
}

pub trait AsJson {
    fn to_json(&self) -> Result<Json, AsJsonError>;
    fn from_json(json: &Json) -> Result<Self, AsJsonError> where Self: Sized;
}
