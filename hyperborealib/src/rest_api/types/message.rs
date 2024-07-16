use std::str::FromStr;

use serde_json::{json, Value as Json};

use crate::crypto::prelude::*;
use crate::rest_api::{AsJson, AsJsonError};

use super::{MessageEncoding, MessagesError};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Message body.
/// 
/// This is a standard type declared in the
/// hyperborea protocol's paper.
pub struct Message {
    pub content: String,
    pub sign: String,
    pub encoding: MessageEncoding
}

impl Message {
    #[inline]
    /// Create new message from raw values.
    /// 
    /// This methodd will not perform any additional
    /// permutations with given data. It is expected
    /// that they're already done by the user.
    /// 
    /// - `content` must contain encoded content of the message.
    /// 
    /// - `sign` must contain encoded digital signature
    ///   of the original message's content.
    /// 
    /// - `encoding` must contain encoding format of the message.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use std::str::FromStr;
    /// 
    /// use hyperborealib::crypto::prelude::*;
    /// use hyperborealib::rest_api::prelude::*;
    /// 
    /// let sender = SecretKey::random();
    /// let receiver = SecretKey::random().public_key();
    /// 
    /// let encoding = MessageEncoding::from_str("base64/aes256-gcm/deflate").unwrap();
    /// let level = CompressionLevel::Quality;
    /// 
    /// let secret = b"pre-defined 32 bytes secret key.";
    /// let message = b"Hello, World!";
    /// 
    /// let sign = sender.create_signature(message);
    /// 
    /// let content = encoding.forward(message, secret, level).unwrap();
    /// let sign = encoding.forward(sign, secret, level).unwrap();
    /// 
    /// let message = Message::new(content, sign, encoding);
    /// ```
    pub fn new(content: impl ToString, sign: impl ToString, encoding: MessageEncoding) -> Self {
        Self {
            content: content.to_string(),
            sign: sign.to_string(),
            encoding
        }
    }

    /// Build new message.
    /// 
    /// This method will compress, encrypt and encode any input
    /// binary data according to provided `encoding` attributes.
    /// 
    /// Signature and shared secret key is calculated using `sender`
    /// and `receiver` keys.
    /// 
    /// - `sender` must contain reference to the secret key
    ///   of the message's sender. It will be used to create
    ///   digital signature of this message and calculate
    ///   shared secret key for the encryption.
    /// 
    /// - `receiver` must contain reference to the public key
    ///   of the message's receiver. It will be used to calculate
    ///   shared secret key used for data encryption.
    /// 
    /// - `data` should contain the message's content.
    /// 
    /// - `encoding` must contain the message's encoding format.
    /// 
    /// - `level` must contain data compression level.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use std::str::FromStr;
    /// 
    /// use hyperborealib::crypto::prelude::*;
    /// use hyperborealib::rest_api::prelude::*;
    /// 
    /// let sender = SecretKey::random();
    /// let receiver = SecretKey::random().public_key();
    /// 
    /// let encoding = MessageEncoding::from_str("base64/aes256-gcm/deflate").unwrap();
    /// let level = CompressionLevel::Quality;
    /// 
    /// let message = Message::create(
    ///     &sender,
    ///     &receiver,
    ///     b"Hello, World!",
    ///     encoding,
    ///     level
    /// ).unwrap();
    /// ```
    pub fn create(sender: &SecretKey, receiver: &PublicKey, data: impl AsRef<[u8]>, encoding: MessageEncoding, level: CompressionLevel) -> Result<Self, MessagesError> {
        let secret = sender.create_shared_secret(receiver, None);

        let sign = sender.create_signature(data.as_ref());

        Ok(Self {
            content: encoding.forward(data, &secret, level)?,
            sign: encoding.forward(sign, &secret, level)?,
            encoding
        })
    }

    /// Read decoded message's content.
    /// 
    /// This method will decrypt, decompress and decode stored
    /// message's content and validate its signature.
    /// 
    /// - `receiver` must contain reference to the secret key
    ///   of the message's receiver. It will be used
    ///   to calculate shared secret key.
    /// 
    /// - `sender` must contain reference to the public key
    ///   of the message's sender. It will be used to calculate
    ///   shared secret key and verify the message's signature.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use std::str::FromStr;
    /// 
    /// use hyperborealib::crypto::prelude::*;
    /// use hyperborealib::rest_api::prelude::*;
    /// 
    /// let sender = SecretKey::random();
    /// let receiver = SecretKey::random();
    /// 
    /// let encoding = MessageEncoding::from_str("base64/aes256-gcm/deflate").unwrap();
    /// 
    /// // Create the message (compress, encrypt and sign)
    /// let message = Message::create(
    ///     &sender,
    ///     &receiver.public_key(),
    ///     b"Hello, World!",
    ///     encoding,
    ///     CompressionLevel::default()
    /// ).unwrap();
    /// 
    /// // Read the message (decompress, decrypt and verify signature)
    /// let content = message.read(&receiver, &sender.public_key()).unwrap();
    /// 
    /// assert_eq!(content, b"Hello, World!");
    /// ```
    pub fn read(&self, receiver: &SecretKey, sender: &PublicKey) -> Result<Vec<u8>, MessagesError> {
        let secret = receiver.create_shared_secret(sender, None);

        let content = self.encoding.backward(&self.content, &secret)?;
        let sign = self.encoding.backward(&self.sign, &secret)?;

        if !sender.verify_signature(&content, sign)? {
            return Err(MessagesError::InvalidMessageSignature);
        }

        Ok(content)
    }
}

impl AsJson for Message {
    fn to_json(&self) -> Result<Json, AsJsonError> {
        Ok(json!({
            "content": self.content,
            "sign": self.sign,
            "encoding": self.encoding.to_string()
        }))
    }

    fn from_json(json: &Json) -> Result<Self, AsJsonError> where Self: Sized {
        Ok(Self {
            content: json.get("content")
                .and_then(Json::as_str)
                .map(String::from)
                .ok_or_else(|| AsJsonError::FieldNotFound("content"))?,

            sign: json.get("sign")
                .and_then(Json::as_str)
                .map(String::from)
                .ok_or_else(|| AsJsonError::FieldNotFound("sign"))?,

            encoding: json.get("encoding")
                .and_then(Json::as_str)
                .map(MessageEncoding::from_str)
                .ok_or_else(|| AsJsonError::FieldNotFound("encoding"))?
                .map_err(|format| AsJsonError::Other(format!("Field 'encoding' contained invalid message encoding format: '{format}'").into()))?
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::rest_api::types::message_encoding::tests::get_encodings;

    use super::*;

    #[test]
    fn serialize() -> Result<(), AsJsonError> {
        let sender = SecretKey::random();
        let receiver = SecretKey::random();

        for encoding in get_encodings().unwrap() {
            let message = Message::create(
                &sender,
                &receiver.public_key(),
                b"Hello, World!",
                encoding,
                CompressionLevel::default()
            ).unwrap();

            assert_eq!(Message::from_json(&message.to_json()?)?, message);
        }

        Ok(())
    }

    #[test]
    fn create_read() -> Result<(), MessagesError> {
        let sender = SecretKey::random();
        let receiver = SecretKey::random();

        for encoding in get_encodings()? {
            let message = Message::create(
                &sender,
                &receiver.public_key(),
                b"Hello, World!",
                encoding,
                CompressionLevel::default()
            )?;

            assert_eq!(message.read(&receiver, &sender.public_key())?, b"Hello, World!");
        }

        Ok(())
    }
}
