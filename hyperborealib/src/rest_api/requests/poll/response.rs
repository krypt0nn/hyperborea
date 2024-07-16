use serde_json::{json, Value as Json};

use crate::rest_api::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// `POST /api/v1/poll` response body.
/// 
/// Refer to `PollResponse` for details.
pub struct PollResponseBody {
    pub messages: Vec<MessageInfo>,
    pub remaining: u64
}

impl PollResponseBody {
    #[inline]
    /// Create new `POST /api/v1/poll` response body.
    /// 
    /// - `messages` must be a vector of messages info
    ///   stored in the server inbox for the requester client.
    /// 
    /// - `remaining` must be a number of remaining inbox messages.
    pub fn new(messages: impl Into<Vec<MessageInfo>>, remaining: u64) -> Self {
        Self {
            messages: messages.into(),
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
    use std::str::FromStr;

    use crate::crypto::asymmetric::SecretKey;

    use super::*;

    #[test]
    fn serialize() -> Result<(), AsJsonError> {
        let client = SecretKey::random();
        let server = SecretKey::random();

        let certificate = ConnectionCertificate::new(&client, server.public_key());
        let info = ClientInfo::thin();

        let client = Client::new(client.public_key(), certificate, info);
        let server = Server::new(server.public_key(), "example.org");

        let sender = Sender::new(client, server.clone());

        let encoding = MessageEncoding::from_str("base64").unwrap();
        let message = Message::new("content", "sign", encoding);

        let info = MessageInfo::now(sender, "Hello, World!", message);

        let response = PollResponseBody::new(vec![info], 100);

        assert_eq!(PollResponseBody::from_json(&response.to_json()?)?, response);

        Ok(())
    }
}
