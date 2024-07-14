use serde_json::{json, Value as Json};

use crate::crypto::asymmetric::PublicKey;

use crate::rest_api::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SendRequestBody {
    pub sender: Sender,
    pub receiver_public: PublicKey,
    pub channel: String,
    pub message: Message
}

impl SendRequestBody {
    #[inline]
    pub fn new(sender: Sender, receiver_public: PublicKey, channel: impl ToString, message: Message) -> Self {
        Self {
            sender,
            receiver_public,
            channel: channel.to_string(),
            message
        }
    }
}

impl AsJson for SendRequestBody {
    fn to_json(&self) -> Result<Json, AsJsonError> {
        Ok(json!({
            "sender": self.sender.to_json()?,
            "receiver": {
                "public_key": self.receiver_public.to_base64()
            },
            "channel": self.channel,
            "message": self.message.to_json()?
        }))
    }

    fn from_json(json: &Json) -> Result<Self, AsJsonError> where Self: Sized {
        let Some(receiver) = json.get("receiver") else {
            return Err(AsJsonError::FieldNotFound("receiver"));
        };

        Ok(Self {
            sender: json.get("sender")
                .map(Sender::from_json)
                .ok_or_else(|| AsJsonError::FieldNotFound("sender"))??,

            receiver_public: receiver.get("public_key")
                .and_then(Json::as_str)
                .ok_or_else(|| AsJsonError::FieldNotFound("receiver.public_key"))
                .map(PublicKey::from_base64)??,

            channel: json.get("channel")
                .and_then(Json::as_str)
                .map(String::from)
                .ok_or_else(|| AsJsonError::FieldNotFound("channel"))?,

            message: json.get("message")
                .map(Message::from_json)
                .ok_or_else(|| AsJsonError::FieldNotFound("message"))??

        })
    }
}

#[cfg(test)]
mod tests {
    use crate::crypto::asymmetric::SecretKey;
    use crate::rest_api::prelude::*;

    use super::*;

    #[test]
    fn serialize() -> Result<(), AsJsonError> {
        let client = SecretKey::random();
        let server = SecretKey::random();

        let info = ClientInfo::thin();
        let cert = ConnectionCertificate::new(&client, server.public_key());

        let client = Client::new(client.public_key(), cert, info);
        let server = Server::new(server.public_key(), "amogus");

        let sender = Sender::new(client, server.clone());

        let message_encoding = MessageEncoding::from_str("base64/plain").unwrap();
        let message = Message::new("content", "sign", message_encoding);

        let request = SendRequestBody::new(sender, server.public_key, "amogus", message);

        assert_eq!(SendRequestBody::from_json(&request.to_json()?)?, request);

        Ok(())
    }
}
