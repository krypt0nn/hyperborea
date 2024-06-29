use serde_json::{json, Value as Json};

use crate::crypto::{PublicKey, SecretKey};
use crate::rest_api::{AsJson, AsJsonError};

use super::{MessageEncoding, Error};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Message {
    pub content: String,
    pub sign: String,
    pub encoding: MessageEncoding
}

impl Message {
    #[inline]
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
    pub fn create(sender: &SecretKey, receiver: &PublicKey, data: impl AsRef<[u8]>, encoding: MessageEncoding) -> Result<Self, Error> {
        let secret = sender.create_shared_secret(receiver, None);

        let sign = sender.create_signature(data.as_ref());

        Ok(Self {
            content: encoding.forward(data, secret)?,
            sign: encoding.forward(sign, secret)?,
            encoding
        })
    }

    /// Read decoded message's content.
    /// 
    /// This method will decrypt, decompress and decode stored
    /// message's content and validate its signature.
    pub fn read(&self, receiver: &SecretKey, sender: &PublicKey) -> Result<Vec<u8>, Error> {
        let secret = receiver.create_shared_secret(sender, None);

        let content = self.encoding.backward(&self.content, secret)?;
        let sign = self.encoding.backward(&self.sign, secret)?;

        if !sender.verify_signature(&content, sign)? {
            return Err(Error::InvalidMessageSignature);
        }

        Ok(content)
    }
}

impl AsJson for Message {
    fn to_json(&self) -> Result<Json, AsJsonError> {
        Ok(json!({
            "content": self.content,
            "sign": self.sign,
            "encoding": self.encoding.to_str()
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
    use super::*;

    fn encodings() -> Result<[MessageEncoding; 6], Error> {
        Ok([
            MessageEncoding::from_str("base64/plain")?,
            MessageEncoding::from_str("base64/deflate")?,
            MessageEncoding::from_str("base64/aes256-gcm")?,
            MessageEncoding::from_str("base64/chacha20-poly1305")?,
            MessageEncoding::from_str("base64/aes256-gcm/deflate")?,
            MessageEncoding::from_str("base64/chacha20-poly1305/deflate")?
        ])
    }

    #[test]
    fn serialize() -> Result<(), AsJsonError> {
        let sender = SecretKey::random();
        let receiver = SecretKey::random();

        for encoding in encodings().unwrap() {
            let message = Message::create(
                &sender,
                &receiver.public_key(),
                b"Hello, World!",
                encoding
            ).unwrap();

            assert_eq!(Message::from_json(&message.to_json()?)?, message);
        }

        Ok(())
    }

    #[test]
    fn create_read() -> Result<(), Error> {
        let sender = SecretKey::random();
        let receiver = SecretKey::random();

        for encoding in encodings().unwrap() {
            let message = Message::create(
                &sender,
                &receiver.public_key(),
                b"Hello, World!",
                encoding
            ).unwrap();

            assert_eq!(message.read(&receiver, &sender.public_key())?, b"Hello, World!");
        }

        Ok(())
    }
}
