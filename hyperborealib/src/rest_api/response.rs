use serde_json::{json, Value as Json};

use crate::crypto::{
    PublicKey,
    base64_encode,
    base64_decode
};

use crate::STANDARD_VERSION;

use super::status::ResponseStatus;

use super::{
    AsJson,
    AsJsonError,
    ValidationError
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Response<T> {
    Success {
        standard: u64,
        status: ResponseStatus,
        public_key: PublicKey,
        proof_sign: Vec<u8>,
        response: T
    },

    Error {
        standard: u64,
        status: ResponseStatus,
        reason: String
    }
}

impl<T> Response<T> {
    pub fn success(status: ResponseStatus, public_key: PublicKey, proof: impl Into<Vec<u8>>, response: T) -> Self {
        Self::Success {
            standard: STANDARD_VERSION,
            status,
            public_key,
            proof_sign: proof.into(),
            response
        }
    }

    pub fn error(status: ResponseStatus, reason: impl ToString) -> Self {
        Self::Error {
            standard: STANDARD_VERSION,
            status,
            reason: reason.to_string()
        }
    }

    pub fn standard(&self) -> u64 {
        match self {
            Self::Success { standard, .. } |
            Self::Error { standard, .. } => *standard
        }
    }

    pub fn status(&self) -> ResponseStatus {
        match self {
            Self::Success { status, .. } |
            Self::Error { status, .. } => *status
        }
    }

    pub fn validate(&self, proof_seed: u64) -> Result<bool, ValidationError> {
        match self {
            Self::Success { public_key, proof_sign, .. } => {
                if proof_seed < 1 << 63 {
                    return Err(ValidationError::InvalidSeed);
                }
        
                Ok(public_key.verify_signature(
                    proof_seed.to_be_bytes(),
                    proof_sign
                )?)
            }

            Self::Error { .. } => Ok(true)
        }
    }
}

impl<T: AsJson> AsJson for Response<T> {
    fn to_json(&self) -> Result<Json, AsJsonError> {
        let value = match self {
            Self::Success { standard, status, public_key, proof_sign, response } => {
                match standard {
                    1 => json!({
                        "standard": standard,
                        "status": status.to_code(),
                        "public_key": public_key.to_base64(),
                        "proof": {
                            "sign": base64_encode(proof_sign)
                        },
                        "response": response.to_json()?
                    }),

                    _ => return Err(AsJsonError::InvalidStandard(*standard))
                }
            }

            Self::Error { standard, status, reason } => {
                match standard {
                    1 => json!({
                        "standard": standard,
                        "status": status.to_code(),
                        "reason": reason
                    }),

                    _ => return Err(AsJsonError::InvalidStandard(*standard))
                }
            }
        };

        Ok(value)
    }

    fn from_json(json: &Json) -> Result<Self, AsJsonError> where Self: Sized {
        let Some(standard) = json.get("standard").and_then(Json::as_u64) else {
            return Err(AsJsonError::FieldNotFound("standard"));
        };

        match standard {
            1 => {
                let Some(status) = json.get("status") else {
                    return Err(AsJsonError::FieldNotFound("status"));
                };

                let Some(status) = status.as_u64().and_then(ResponseStatus::from_code) else {
                    return Err(AsJsonError::FieldValueInvalid("status"));
                };

                if status.is_success() {
                    let Some(public_key) = json.get("public_key").and_then(Json::as_str) else {
                        return Err(AsJsonError::FieldNotFound("public_key"));
                    };
    
                    let Some(proof) = json.get("proof") else {
                        return Err(AsJsonError::FieldNotFound("proof"));
                    };
    
                    let Some(proof_sign) = proof.get("sign").and_then(Json::as_str) else {
                        return Err(AsJsonError::FieldNotFound("proof.sign"));
                    };
    
                    let Some(response) = json.get("response") else {
                        return Err(AsJsonError::FieldNotFound("response"));
                    };

                    Ok(Self::Success {
                        standard,
                        status,
                        public_key: PublicKey::from_base64(public_key)?,
                        proof_sign: base64_decode(proof_sign)?,
                        response: T::from_json(response)?
                    })
                }

                else {
                    let Some(reason) = json.get("reason").and_then(Json::as_str) else {
                        return Err(AsJsonError::FieldNotFound("reason"));
                    };

                    Ok(Self::Error {
                        standard,
                        status,
                        reason: reason.to_string()
                    })
                }
            }

            _ => Err(AsJsonError::InvalidStandard(standard))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize_success() -> Result<(), AsJsonError> {
        use crate::crypto::SecretKey;
        use crate::rest_api::connect::ConnectResponse;

        let secret = SecretKey::random();
        let public = SecretKey::random().public_key();

        let connect_response = ConnectResponse::success(ResponseStatus::Success, &secret, 123456789);

        let proof = secret.create_signature(123456789_u64.to_be_bytes());

        let response = Response::success(ResponseStatus::Success, public, proof, connect_response);

        assert_eq!(Response::from_json(&response.to_json()?)?, response);

        Ok(())
    }

    #[test]
    fn serialize_error() -> Result<(), AsJsonError> {
        use crate::rest_api::connect::ConnectResponse;

        let response = Response::<ConnectResponse>::error(ResponseStatus::MessageTooLarge, "Hello, World!");

        assert_eq!(Response::from_json(&response.to_json()?)?, response);

        Ok(())
    }
}
