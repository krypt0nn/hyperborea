use serde_json::{json, Value as Json};

use crate::rest_api::{AsJson, AsJsonError};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// `POST /api/v1/announce` response body.
/// 
/// Refer to `AnnounceResponse` for details.
pub struct AnnounceResponseBody;

impl AnnounceResponseBody {
    #[inline]
    #[allow(clippy::new_without_default)]
    /// Create announce response body.
    /// 
    /// It doesn't contain any important info
    /// so everything is filled automatically.
    pub fn new() -> Self {
        Self
    }
}

impl AsJson for AnnounceResponseBody {
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
        let response = AnnounceResponseBody;

        assert_eq!(AnnounceResponseBody::from_json(&response.to_json()?)?, response);

        Ok(())
    }
}
