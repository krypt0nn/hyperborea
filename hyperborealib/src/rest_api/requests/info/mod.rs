use serde_json::{json, Value as Json};

use crate::crypto::prelude::*;
use crate::rest_api::prelude::*;

use crate::STANDARD_VERSION;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// `GET /api/v1/info` response.
/// 
/// This API should provide general information about
/// the protocol's server. This information
/// can be used by clients to choose whether they should
/// connect to this server, or to get this server's public key
/// for further API calls.
pub struct InfoResponse {
    pub standard: u64,
    pub public_key: PublicKey,
    pub proof_seed: u64,
    pub proof_sign: Vec<u8>,

    // TODO: stats
}

impl InfoResponse {
    pub fn new(server_secret: &SecretKey) -> Self {
        let proof_seed = safe_random_u64_long();

        let proof_sign = server_secret.create_signature(proof_seed.to_be_bytes());

        Self {
            standard: STANDARD_VERSION,
            public_key: server_secret.public_key(),
            proof_seed,
            proof_sign
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

impl AsJson for InfoResponse {
    fn to_json(&self) -> Result<Json, AsJsonError> {
        match self.standard {
            1 => Ok(json!({
                "standard": self.standard,
                "server": {
                    "public_key": self.public_key.to_base64(),
                },
                "proof": {
                    "seed": self.proof_seed,
                    "sign": base64_encode(&self.proof_sign)
                }
            })),

            _ => Err(AsJsonError::InvalidStandard(self.standard))
        }
    }

    fn from_json(json: &Json) -> Result<Self, AsJsonError> where Self: Sized {
        let Some(standard) = json.get("standard").and_then(Json::as_u64) else {
            return Err(AsJsonError::FieldNotFound("standard"));
        };

        match standard {
            1 => {
                let Some(server) = json.get("server") else {
                    return Err(AsJsonError::FieldNotFound("server"));
                };

                let Some(public_key) = server.get("public_key").and_then(Json::as_str) else {
                    return Err(AsJsonError::FieldNotFound("server.public_key"));
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

                Ok(Self {
                    standard,
                    public_key: PublicKey::from_base64(public_key)?,
                    proof_seed,
                    proof_sign: base64_decode(proof_sign)?
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
        let response = InfoResponse::new(&SecretKey::random());

        assert_eq!(InfoResponse::from_json(&response.to_json()?)?, response);

        Ok(())
    }
}
