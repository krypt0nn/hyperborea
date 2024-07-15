//! Implementations of the structs from the protocol's paper.
//! 
//! All types in this module are made to be used
//! in network communication as part of the
//! requests and responses. They all implement AsJson trait
//! to serialize their content.
//! 
//! They are not intended to be used outside of the library's REST API
//! and may luck many helpful service methods. You should declare
//! your own types or use some abstractions over this low-level library.

use crate::crypto::Error as CryptographyError;

pub(crate) mod client_type;
pub(crate) mod client_info;
pub(crate) mod connection_token;
pub(crate) mod connection_certificate;
pub(crate) mod client;
pub(crate) mod server;
pub(crate) mod message_info;
pub(crate) mod message_encoding;
pub(crate) mod sender;
pub(crate) mod message;

pub use client_type::*;
pub use client_info::*;
pub use connection_token::*;
pub use connection_certificate::*;
pub use client::*;
pub use server::*;
pub use message_info::*;
pub use message_encoding::*;
pub use sender::*;
pub use message::*;

#[derive(Debug, thiserror::Error)]
pub enum MessagesError {
    #[error("Wrong message encoding format provided: '{0}'")]
    WrongMessageEncodingFormat(String),

    #[error("Message's signature is invalid")]
    InvalidMessageSignature,

    #[error(transparent)]
    CryptographyError(#[from] CryptographyError)
}
