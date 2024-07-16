use serde_json::{json, Value as Json};

use crate::rest_api::prelude::*;

use crate::time::timestamp;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Information about the message (its header).
/// 
/// This is a standard type declared in the
/// hyperborea protocol's paper.
pub struct MessageInfo {
    pub sender: Sender,
    pub channel: String,
    pub message: Message,
    pub received_at: u64
}

impl MessageInfo {
    #[inline]
    /// Create new stored message info.
    /// 
    /// - `sender` must be information about the sender
    ///   client and server it is connected to.
    /// 
    /// - `channel` must be a name of the channel this
    ///   message was sent into.
    /// 
    /// - `message` must contain the message's body.
    /// 
    /// - `received_at` must contain message receiving timestamp.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use std::str::FromStr;
    /// 
    /// use hyperborealib::crypto::prelude::*;
    /// use hyperborealib::rest_api::prelude::*;
    /// 
    /// use hyperborealib::time::timestamp;
    /// 
    /// let sender_client_secret   = SecretKey::random();
    /// let sender_server_secret   = SecretKey::random();
    /// let receiver_client_secret = SecretKey::random();
    /// 
    /// // Prepare Sender struct
    /// let certificate = ConnectionCertificate::new(
    ///     &sender_client_secret,
    ///     sender_server_secret.public_key()
    /// );
    /// 
    /// let client = Client::new(
    ///     sender_client_secret.public_key(),
    ///     certificate,
    ///     ClientInfo::thin()
    /// );
    /// 
    /// let server = Server::new(
    ///     sender_server_secret.public_key(),
    ///     "example.org"
    /// );
    /// 
    /// let sender = Sender::new(client, server);
    /// 
    /// // Prepare message
    /// let encoding = MessageEncoding::from_str("base64/chacha20-poly1305").unwrap();
    /// 
    /// let message = Message::create(
    ///     &sender_client_secret,
    ///     &receiver_client_secret.public_key(),
    ///     b"Hello, World!",
    ///     encoding,
    ///     CompressionLevel::default()
    /// ).unwrap();
    /// 
    /// // Prepare message info
    /// let message_info = MessageInfo::new(
    ///     sender,
    ///     "example channel",
    ///     message,
    ///     timestamp()
    /// );
    /// ```
    pub fn new(sender: Sender, channel: impl ToString, message: Message, received_at: u64) -> Self {
        Self {
            sender,
            channel: channel.to_string(),
            message,
            received_at
        }
    }

    #[inline]
    /// Run `new()` method with current timestamp.
    pub fn now(sender: Sender, channel: impl ToString, message: Message) -> Self {
        Self::new(sender, channel, message, timestamp())
    }
}

impl AsJson for MessageInfo {
    fn to_json(&self) -> Result<serde_json::Value, AsJsonError> {
        Ok(json!({
            "sender": self.sender.to_json()?,
            "channel": self.channel,
            "message": self.message.to_json()?,
            "received_at": self.received_at
        }))
    }

    fn from_json(json: &Json) -> Result<Self, AsJsonError> where Self: Sized {
        Ok(Self {
            sender: json.get("sender")
                .map(Sender::from_json)
                .ok_or_else(|| AsJsonError::FieldNotFound("sender"))??,

            channel: json.get("channel")
                .and_then(Json::as_str)
                .map(String::from)
                .ok_or_else(|| AsJsonError::FieldNotFound("channel"))?,

            message: json.get("message")
                .map(Message::from_json)
                .ok_or_else(|| AsJsonError::FieldNotFound("message"))??,

            received_at: json.get("received_at")
                .and_then(Json::as_u64)
                .ok_or_else(|| AsJsonError::FieldNotFound("received_at"))?
        })
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use std::str::FromStr;

    use crate::rest_api::types::sender::tests::get_sender;

    use super::*;

    pub fn get_message_info() -> MessageInfo {
        let encoding = MessageEncoding::from_str("base64").unwrap();
        let message = Message::new("content", "sign", encoding);

        MessageInfo::now(get_sender(), "Hello, World!", message)
    }

    #[test]
    fn serialize() -> Result<(), AsJsonError> {
        let message_info = get_message_info();

        assert_eq!(MessageInfo::from_json(&message_info.to_json()?)?, message_info);

        Ok(())
    }
}
