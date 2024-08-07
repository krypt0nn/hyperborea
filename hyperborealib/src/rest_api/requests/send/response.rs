use serde_json::{json, Value as Json};

use crate::rest_api::{AsJson, AsJsonError};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// `POST /api/v1/send` response body.
/// 
/// Refer to `SendResponse` for details.
pub struct SendResponseBody;

impl SendResponseBody {
    #[inline]
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self
    }
}

impl AsJson for SendResponseBody {
    fn to_json(&self) -> Result<Json, AsJsonError> {
        Ok(json!({}))
    }

    fn from_json(_json: &Json) -> Result<Self, AsJsonError> where Self: Sized {
        Ok(Self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize() -> Result<(), AsJsonError> {
        let response = SendResponseBody;

        assert_eq!(SendResponseBody::from_json(&response.to_json()?)?, response);

        Ok(())
    }
}
