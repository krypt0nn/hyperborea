use serde_json::{json, Value as Json};

use crate::crypto::prelude::*;

use crate::STANDARD_VERSION;

use super::{
    AsJson,
    AsJsonError,
    ValidationError
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Protocol's REST API requests header.
/// 
/// According to the standard all the `POST` requests
/// must follow the same top-level structure
/// (contain a proper header) which is used to verify
/// that the request is sent by a client with specified
/// public key.
pub struct Request<T> {
    pub standard: u64,
    pub public_key: PublicKey,
    pub proof_seed: u64,
    pub proof_sign: Vec<u8>,
    pub request: T
}

impl<T> Request<T> {
    /// Create new REST API request.
    /// 
    /// - `client_secret` must contain reference to
    ///   the secret key of the request's sender.
    ///   It is used to sign random number to validate
    ///   this request.
    /// 
    /// - `request` can contain any value, preferably
    ///   implementing `AsJson` trait.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use hyperborealib::crypto::prelude::*;
    /// use hyperborealib::rest_api::prelude::*;
    /// 
    /// let request = Request::new(&SecretKey::random(), ());
    /// ```
    pub fn new(client_secret: &SecretKey, request: T) -> Self {
        let proof_seed = safe_random_u64_long();

        let proof_sign = client_secret.create_signature(proof_seed.to_be_bytes());

        Self {
            standard: STANDARD_VERSION,
            public_key: client_secret.public_key(),
            proof_seed,
            proof_sign,
            request
        }
    }

    /// Validate that the request's header is correct.
    /// 
    /// This method will verify that the proof signature
    /// is signed correctly by the sender using given public key.
    /// 
    /// This method will also verify that the proof seed
    /// is correctly chosen (`>= 1^63`). This is important
    /// for signature generation to not to have many zero bytes.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use hyperborealib::crypto::prelude::*;
    /// use hyperborealib::rest_api::prelude::*;
    /// 
    /// // Create random secret key
    /// let secret_key = SecretKey::random();
    /// 
    /// // Create empty request
    /// let request = Request::new(&secret_key, ());
    /// 
    /// assert!(request.validate().unwrap());
    /// ```
    pub fn validate(&self) -> Result<bool, ValidationError> {
        if self.proof_seed < 1 << 63 {
            return Err(ValidationError::InvalidSeed);
        }

        Ok(self.public_key.verify_signature(
            self.proof_seed.to_be_bytes(),
            &self.proof_sign
        )?)
    }
}

impl<T: AsJson> AsJson for Request<T> {
    fn to_json(&self) -> Result<Json, AsJsonError> {
        let value = match self.standard {
            1 => json!({
                "standard": self.standard,
                "public_key": self.public_key.to_base64(),
                "proof": {
                    "seed": self.proof_seed,
                    "sign": base64_encode(&self.proof_sign)
                },
                "request": self.request.to_json()?
            }),

            _ => return Err(AsJsonError::InvalidStandard(self.standard))
        };

        Ok(value)
    }

    fn from_json(json: &Json) -> Result<Self, AsJsonError> where Self: Sized {
        let Some(standard) = json.get("standard").and_then(Json::as_u64) else {
            return Err(AsJsonError::FieldNotFound("standard"));
        };

        match standard {
            1 => {
                let Some(public_key) = json.get("public_key").and_then(Json::as_str) else {
                    return Err(AsJsonError::FieldNotFound("public_key"));
                };

                let Some(proof) = json.get("proof") else {
                    return Err(AsJsonError::FieldNotFound("proof"));
                };

                let Some(proof_seed) = proof.get("seed").and_then(Json::as_u64) else {
                    return Err(AsJsonError::FieldNotFound("proof.seed"));
                };

                let Some(proof_sign) = proof.get("sign").and_then(Json::as_str) else {
                    return Err(AsJsonError::FieldNotFound("proof.sign"));
                };

                let Some(request) = json.get("request") else {
                    return Err(AsJsonError::FieldNotFound("request"));
                };

                Ok(Self {
                    standard,
                    public_key: PublicKey::from_base64(public_key)?,
                    proof_seed,
                    proof_sign: base64_decode(proof_sign)?,
                    request: T::from_json(request)?
                })
            }

            _ => Err(AsJsonError::InvalidStandard(standard))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::rest_api::requests::ConnectRequest;
    use crate::rest_api::types::ClientInfo;

    use super::*;

    #[test]
    fn serialize() -> Result<(), AsJsonError> {
        let secret = SecretKey::random();
        let public = SecretKey::random().public_key();

        let connect_request = ConnectRequest::new(
            &secret,
            public,
            ClientInfo::thin()
        );

        let request = Request::new(&secret, connect_request);

        assert_eq!(Request::from_json(&request.to_json()?)?, request);

        Ok(())
    }

    #[test]
    fn validate() -> Result<(), ValidationError> {
        let secret_key = SecretKey::random();

        // Valid request header

        let request = Request::new(&secret_key, ());

        assert!(request.validate()?);

        // Invalid proof seed

        let mut request = Request::new(&secret_key, ());

        request.proof_seed = 0;

        assert!(request.validate().is_err());

        // Invalid sign (different proof seed)

        let mut request = Request::new(&secret_key, ());

        request.proof_seed = safe_random_u64_long();

        assert!(!request.validate()?);

        // Invalid sign (different proof sign)

        let mut request = Request::new(&secret_key, ());

        request.proof_sign = vec![1, 2, 3, 4, 5, 6, 7, 8];

        assert!(request.validate().is_err());

        Ok(())
    }
}
