use serde_json::{json, Value as Json};

use crate::crypto::prelude::*;

use crate::STANDARD_VERSION;

use super::status::ResponseStatus;

use super::{
    AsJson,
    AsJsonError,
    ValidationError
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Protocol's REST API response header.
/// 
/// According to the standard all the `POST` requests
/// responses must follow the same top-level structure
/// (contain a proper header) which is used to verify
/// that the response is sent by a server with specified
/// public key, and to identify the request's status.
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
    /// Create successful response.
    /// 
    /// - `status` must contain status code of the response
    ///   (`100 Success` in most cases).
    /// 
    /// - `public_key` must contain public key of the responder.
    /// 
    /// - `proof` must contain signature of the original request's
    ///   proof seed signed by the responder's secret key
    ///   (linked with the `public_key`).
    /// 
    /// - `response` can contain any value, preferably
    ///   implementing `AsJson` trait.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use hyperborealib::crypto::prelude::*;
    /// use hyperborealib::rest_api::prelude::*;
    /// 
    /// let secret_key = SecretKey::random();
    /// 
    /// let proof_seed = safe_random_u64_long();
    /// let proof_sign = secret_key.create_signature(proof_seed.to_be_bytes());
    /// 
    /// let response = Response::success(
    ///     ResponseStatus::Success,
    ///     secret_key.public_key(),
    ///     proof_sign,
    ///     ()
    /// );
    /// ```
    pub fn success(status: ResponseStatus, public_key: PublicKey, proof: impl Into<Vec<u8>>, response: T) -> Self {
        Self::Success {
            standard: STANDARD_VERSION,
            status,
            public_key,
            proof_sign: proof.into(),
            response
        }
    }

    /// Create error response.
    /// 
    /// - `status` must contain status code of the response.
    /// 
    /// - `reason` must contain the explanation string of the error
    ///   (error message and/or description).
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use hyperborealib::rest_api::prelude::*;
    /// 
    /// let response = Response::error(
    ///     ResponseStatus::ServerError,
    ///     "Example error"
    /// );
    /// ```
    pub fn error(status: ResponseStatus, reason: impl ToString) -> Self {
        Self::Error {
            standard: STANDARD_VERSION,
            status,
            reason: reason.to_string()
        }
    }

    /// Get `standard` field from the response header.
    /// 
    /// This is a helper function for easier work
    /// with response enum.
    pub fn standard(&self) -> u64 {
        match self {
            Self::Success { standard, .. } |
            Self::Error { standard, .. } => *standard
        }
    }

    /// Get `status` field from the response header.
    /// 
    /// This is a helper function for easier work
    /// with response enum.
    pub fn status(&self) -> ResponseStatus {
        match self {
            Self::Success { status, .. } |
            Self::Error { status, .. } => *status
        }
    }

    /// Validate that the response is correct (sent by a real server).
    /// 
    /// - `proof_seed` must contain proof seed
    ///   sent in the original request.
    /// 
    /// > Note: error responses are always valid.
    /// 
    /// # Example
    /// 
    /// ## Success response
    /// 
    /// ```rust
    /// use hyperborealib::crypto::prelude::*;
    /// use hyperborealib::rest_api::prelude::*;
    /// 
    /// let secret_key = SecretKey::random();
    /// 
    /// let proof_seed = safe_random_u64_long();
    /// let proof_sign = secret_key.create_signature(proof_seed.to_be_bytes());
    /// 
    /// let response = Response::success(
    ///     ResponseStatus::Success,
    ///     secret_key.public_key(),
    ///     proof_sign,
    ///     ()
    /// );
    /// 
    /// assert!(response.validate(proof_seed).unwrap(), true);
    /// ```
    /// 
    /// ## Error response
    /// 
    /// ```rust
    /// use hyperborealib::rest_api::prelude::*;
    /// 
    /// let response = Response::error(
    ///     ResponseStatus::ServerError,
    ///     "Example error"
    /// );
    /// 
    /// assert!(response.validate(0).unwrap(), true);
    /// ```
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
    use crate::crypto::asymmetric::SecretKey;
    use crate::rest_api::requests::ConnectResponse;

    use super::*;

    #[test]
    fn serialize_success() -> Result<(), AsJsonError> {
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
        let response = Response::<ConnectResponse>::error(ResponseStatus::MessageTooLarge, "Hello, World!");

        assert_eq!(Response::from_json(&response.to_json()?)?, response);

        Ok(())
    }
}
