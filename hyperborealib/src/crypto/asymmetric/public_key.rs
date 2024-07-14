use k256::ecdsa::signature::Verifier;

use crate::crypto::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PublicKey(pub(crate) k256::PublicKey);

impl PublicKey {
    /// Serialize public key into fixed length bytes slice.
    pub fn to_bytes(&self) -> [u8; 33] {
        let bytes = self.0.to_sec1_bytes();

        // k256::PublicKey::to_sec1_bytes() should always return 33 bytes
        // but it's better to be safe
        assert_eq!(bytes.len(), 33);

        let mut result = [0_u8; 33];

        result.copy_from_slice(&bytes);

        result
    }

    /// Deserialize public key from given bytes slice.
    pub fn from_bytes(bytes: impl AsRef<[u8]>) -> Result<Self, CryptographyError> {
        Ok(Self(k256::PublicKey::from_sec1_bytes(bytes.as_ref())?))
    }

    /// Serialize public key into bytes slice and encode it
    /// into base 64 number.
    pub fn to_base64(&self) -> String {
        base64_encode(self.to_bytes())
    }

    /// Decode given base 64 number and deserialize
    /// a public key from it.
    pub fn from_base64(base64: impl AsRef<str>) -> Result<Self, CryptographyError> {
        Self::from_bytes(base64_decode(base64)
            .map_err(|err| CryptographyError::Decoding(err.into()))?)
    }

    /// Verify signature of the message.
    pub fn verify_signature(&self, message: impl AsRef<[u8]>, signature: impl AsRef<[u8]>) -> Result<bool, CryptographyError> {
        let signature = k256::ecdsa::Signature::from_slice(signature.as_ref())?;

        Ok(k256::ecdsa::VerifyingKey::from(&self.0)
            .verify(message.as_ref(), &signature)
            .is_ok())
    }

    /// Decode encapsulated signature, returning the original message if the signature is valid.
    pub fn decode_encapsulated_signature(&self, capsule: impl AsRef<[u8]>) -> Result<Option<Vec<u8>>, CryptographyError> {
        let capsule = capsule.as_ref();

        let mut sign_len_bytes = [0_u8; 8];

        sign_len_bytes.copy_from_slice(&capsule[..8]);

        let sign_len = u64::from_be_bytes(sign_len_bytes) as usize;

        let signature = &capsule[8..sign_len + 8];
        let message = &capsule[sign_len + 8..];

        if !self.verify_signature(message, signature)? {
            return Ok(None);
        }

        Ok(Some(message.to_vec()))
    }
}

impl From<k256::PublicKey> for PublicKey {
    #[inline]
    fn from(value: k256::PublicKey) -> Self {
        Self(value)
    }
}

impl std::hash::Hash for PublicKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.to_bytes().hash(state);
    }
}
