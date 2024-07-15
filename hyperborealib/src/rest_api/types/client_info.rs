use std::str::FromStr;

use serde_json::{json, Value as Json};

use crate::rest_api::{AsJson, AsJsonError};
use crate::rest_api::types::ClientType;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ClientInfo {
    pub client_type: ClientType,
    pub address: Option<String>
}

impl Default for ClientInfo {
    #[inline]
    fn default() -> Self {
        Self {
            client_type: ClientType::Thin,
            address: None
        }
    }
}

impl AsJson for ClientInfo {
    fn to_json(&self) -> Result<Json, AsJsonError> {
        Ok(json!({
            "type": self.client_type.to_string(),
            "address": self.address
        }))
    }

    fn from_json(json: &Json) -> Result<Self, AsJsonError> where Self: Sized {
        Ok(Self {
            client_type: json.get("type")
                .and_then(Json::as_str)
                .map(ClientType::from_str)
                .ok_or_else(|| AsJsonError::FieldNotFound("type"))?
                .map_err(|_| AsJsonError::FieldValueInvalid("client type field contains unknown value"))?,

            address: json.get("address")
                .and_then(Json::as_str)
                .map(String::from)
        })
    }
}

impl ClientInfo {
    #[inline]
    pub fn new(client_type: ClientType, address: Option<impl ToString>) -> Self {
        Self {
            client_type,
            address: address.map(|value| value.to_string())
        }
    }

    #[inline]
    pub fn thin() -> Self {
        Self {
            client_type: ClientType::Thin,
            address: None
        }
    }

    #[inline]
    pub fn thick(address: impl ToString) -> Self {
        Self {
            client_type: ClientType::Thick,
            address: Some(address.to_string())
        }
    }

    #[inline]
    pub fn server(address: impl ToString) -> Self {
        Self {
            client_type: ClientType::Server,
            address: Some(address.to_string())
        }
    }

    #[inline]
    pub fn file(address: impl ToString) -> Self {
        Self {
            client_type: ClientType::File,
            address: Some(address.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize_thin() -> Result<(), AsJsonError> {
        let client = ClientInfo::thin();

        assert_eq!(ClientInfo::from_json(&client.to_json()?)?, client);

        Ok(())
    }

    #[test]
    fn serialize_thick() -> Result<(), AsJsonError> {
        let client = ClientInfo::thick("Hello, World!");

        assert_eq!(ClientInfo::from_json(&client.to_json()?)?, client);

        Ok(())
    }

    #[test]
    fn serialize_server() -> Result<(), AsJsonError> {
        let client = ClientInfo::server("Hello, World!");

        assert_eq!(ClientInfo::from_json(&client.to_json()?)?, client);

        Ok(())
    }

    #[test]
    fn serialize_file() -> Result<(), AsJsonError> {
        let client = ClientInfo::file("Hello, World!");

        assert_eq!(ClientInfo::from_json(&client.to_json()?)?, client);

        Ok(())
    }
}
