use std::str::FromStr;

use serde_json::{json, Value as Json};

use crate::crypto::prelude::*;
use crate::rest_api::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// `POST /api/v1/lookup` request body.
/// 
/// Refer to `LookupRequest` for details.
pub struct LookupRequestBody {
    pub public_key: PublicKey,
    pub client_type: Option<ClientType>
}

impl LookupRequestBody {
    #[inline]
    /// Create new `POST /api/v1/lookup` request body.
    /// 
    /// - `client_public` must contain public key
    ///   of the client we want to find.
    /// 
    /// - `client_type` is an optional filter field
    ///   of the needed client type. It is used to
    ///   split the public keys' namespace.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use hyperborealib::crypto::prelude::*;
    /// use hyperborealib::rest_api::prelude::*;
    /// 
    /// let client_public = SecretKey::random().public_key();
    /// 
    /// let request_body = LookupRequestBody::new(client_public, None);
    /// ```
    pub fn new(client_public: PublicKey, client_type: Option<ClientType>) -> Self {
        Self {
            public_key: client_public,
            client_type
        }
    }
}

impl AsJson for LookupRequestBody {
    fn to_json(&self) -> Result<Json, AsJsonError> {
        Ok(json!({
            "public_key": self.public_key.to_base64(),
            "type": self.client_type.map(|value| value.to_string())
        }))
    }

    fn from_json(json: &Json) -> Result<Self, AsJsonError> where Self: Sized {
        Ok(Self {
            public_key: json.get("public_key")
                .and_then(Json::as_str)
                .ok_or_else(|| AsJsonError::FieldNotFound("public_key"))
                .map(PublicKey::from_base64)??,

            client_type: json.get("type")
                .and_then(Json::as_str)
                .map(ClientType::from_str)
                .transpose()
                .map_err(|_| AsJsonError::FieldValueInvalid("Invalid client type value"))?
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize() -> Result<(), AsJsonError> {
        let public = SecretKey::random().public_key();

        let request = LookupRequestBody::new(public.clone(), Some(ClientType::Thin));

        assert_eq!(LookupRequestBody::from_json(&request.to_json()?)?, request);

        let request = LookupRequestBody::new(public.clone(), None);

        assert_eq!(LookupRequestBody::from_json(&request.to_json()?)?, request);

        Ok(())
    }
}
