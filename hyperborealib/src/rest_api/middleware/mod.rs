use crate::crypto::Error as CryptographyError;
use crate::rest_api::ValidationError;
use crate::rest_api::status::ResponseStatus;

mod client;
mod server;

pub use client::Client;
pub use server::Server;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Proof seed is invalid")]
    InvalidProofSeed,

    #[error("Invalid proof seed signature")]
    InvalidProofSeedSignature,

    #[error(transparent)]
    CryptographyError(#[from] CryptographyError),

    #[error(transparent)]
    SignatureValidationError(#[from] ValidationError),

    #[error("Request failed. Status: {status:?}, reason: {reason}")]
    RequestFailed {
        status: ResponseStatus,
        reason: String
    },

    #[error(transparent)]
    Other(#[from] Box<dyn std::error::Error + Send + Sync>)
}
