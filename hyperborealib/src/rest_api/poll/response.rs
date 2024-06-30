use serde_json::{json, Value as Json};

use crate::rest_api::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PollResponseBody {
    pub messages: Vec<MessageInfo>,
    pub remaining: u64
}

impl PollResponseBody {
    #[inline]
    pub fn new(messages: Vec<MessageInfo>, remaining: u64) -> Self {
        Self {
            messages,
            remaining
        }
    }
}

impl AsJson for PollResponseBody {
    fn to_json(&self) -> Result<Json, AsJsonError> {
        Ok(json!({
            "messages": self.messages.iter()
                .map(MessageInfo::to_json)
                .collect::<Result<Vec<_>, _>>()?,

            "remaining": self.remaining
        }))
    }

    fn from_json(json: &Json) -> Result<Self, AsJsonError> where Self: Sized {
        Ok(Self {
            messages: json.get("messages")
                .and_then(Json::as_array)
                .map(|messages| {
                    messages.iter()
                        .map(MessageInfo::from_json)
                        .collect::<Result<Vec<_>, _>>()
                })
                .ok_or_else(|| AsJsonError::FieldNotFound("messages"))??,

            remaining: json.get("remaining")
                .and_then(Json::as_u64)
                .ok_or_else(|| AsJsonError::FieldNotFound("remaining"))?
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize() -> Result<(), AsJsonError> {
        use crate::crypto::SecretKey;

        let client = SecretKey::random();
        let server = SecretKey::random();

        let info = ClientInfo::thin();
        let cert = ConnectionCertificate::new(&client, server.public_key());

        let client = Client::new(client.public_key(), cert, info);
        let server = Server::new(server.public_key(), "amogus");

        let sender = Sender::new(client, server.clone());

        let encoding = MessageEncoding::from_str("base64/plain").unwrap();
        let message = Message::new("content", "sign", encoding);

        let info = MessageInfo::new_now(sender, "Hello, World!", message);

        let response = PollResponseBody::new(vec![info], 100);

        assert_eq!(PollResponseBody::from_json(&response.to_json()?)?, response);

        Ok(())
    }
}
