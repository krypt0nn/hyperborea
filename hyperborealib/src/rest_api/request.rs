use serde_json::{json, Value as Json};

use crate::crypto::{
    PublicKey,
    SecretKey,
    base64_decode,
    base64_encode,
    safe_random_u64
};

use crate::STANDARD_VERSION;

use super::{
    AsJson,
    AsJsonError,
    ValidationError
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Request<T> {
    pub standard: u64,
    pub public_key: PublicKey,
    pub proof_seed: u64,
    pub proof_sign: Vec<u8>,
    pub request: T
}

impl<T> Request<T> {
    pub fn new(client_secret: &SecretKey, request: T) -> Self {
        // Generate 64 bits long number
        let proof_seed = (1 << 63) + (safe_random_u64() >> 1);

        let proof_sign = client_secret.create_signature(proof_seed.to_be_bytes());

        Self {
            standard: STANDARD_VERSION,
            public_key: client_secret.public_key(),
            proof_seed,
            proof_sign,
            request
        }
    }

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
    use super::*;

    #[test]
    fn serialize() -> Result<(), AsJsonError> {
        use crate::crypto::SecretKey;
        use crate::rest_api::connect::{ConnectRequest, ClientInfo};

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
}
