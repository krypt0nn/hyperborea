use serde_json::{json, Value as Json};

use crate::rest_api::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PollRequestBody {
    pub channel: String,
    pub limit: Option<u64>
}

impl PollRequestBody {
    #[inline]
    pub fn new(channel: impl ToString, limit: Option<u64>) -> Self {
        Self {
            channel: channel.to_string(),
            limit
        }
    }
}

impl AsJson for PollRequestBody {
    fn to_json(&self) -> Result<Json, AsJsonError> {
        Ok(json!({
            "channel": self.channel,
            "limit": self.limit
        }))
    }

    fn from_json(json: &Json) -> Result<Self, AsJsonError> where Self: Sized {
        Ok(Self {
            channel: json.get("channel")
                .and_then(Json::as_str)
                .map(String::from)
                .ok_or_else(|| AsJsonError::FieldNotFound("channel"))?,

            limit: json.get("limit")
                .ok_or_else(|| AsJsonError::FieldNotFound("channel"))
                .and_then(|value| {
                    if value.is_null() {
                        Ok(None)
                    } else {
                        value.as_u64()
                            .map(Some)
                            .ok_or_else(|| AsJsonError::FieldValueInvalid("channel"))
                    }
                })?
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize() -> Result<(), AsJsonError> {
        use crate::rest_api::prelude::*;

        let request = PollRequestBody::new("Hello, World!", None);

        assert_eq!(PollRequestBody::from_json(&request.to_json()?)?, request);

        let request = PollRequestBody::new("Hello, World!", Some(5));

        assert_eq!(PollRequestBody::from_json(&request.to_json()?)?, request);

        Ok(())
    }
}
