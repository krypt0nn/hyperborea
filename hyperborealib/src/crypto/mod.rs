pub mod utils;
pub mod asymmetric;
pub mod encoding;
pub mod compression;
pub mod encryption;

pub mod prelude {
    pub use super::Error as CryptographyError;

    pub use super::utils::*;
    pub use super::asymmetric::*;
    pub use super::encoding::prelude::*;
    pub use super::compression::prelude::*;
    pub use super::encryption::prelude::*;
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    EllipticCurve(#[from] k256::elliptic_curve::Error),

    #[error(transparent)]
    Signature(#[from] k256::ecdsa::Error),

    #[error("Failed to decode data: {0}")]
    Decoding(#[source] Box<dyn std::error::Error + Send + Sync>),

    #[error("Failed to compress data: {0}")]
    Compression(#[source] Box<dyn std::error::Error + Send + Sync>),

    #[error("Failed to decompress data: {0}")]
    Decompression(#[source] Box<dyn std::error::Error + Send + Sync>),

    #[error("Failed to encrypt data: {0}")]
    Encryption(#[source] Box<dyn std::error::Error + Send + Sync>),

    #[error("Failed to decrypt data: {0}")]
    Decryption(#[source] Box<dyn std::error::Error + Send + Sync>)
}
