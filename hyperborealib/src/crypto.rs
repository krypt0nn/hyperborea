use rand_chacha::ChaCha20Rng;
use rand_chacha::rand_core::{CryptoRngCore, SeedableRng, RngCore};

use k256::ecdsa::signature::{Signer, Verifier};

use base64::Engine;
use base64::engine::GeneralPurpose as Base64Engine;

lazy_static::lazy_static! {
    pub static ref BASE64: Base64Engine = Base64Engine::new(
        &base64::alphabet::URL_SAFE,
        base64::engine::GeneralPurposeConfig::default()
    );
}

pub const HKDF_INFO: [u8; 64] = [162, 241, 203, 77, 49, 90, 31, 126, 67, 94, 191, 219, 56, 141, 46, 233, 70, 18, 207, 194, 52, 154, 176, 139, 244, 222, 155, 110, 177, 91, 32, 218, 150, 232, 148, 23, 13, 172, 48, 131, 95, 216, 144, 224, 163, 106, 254, 135, 93, 220, 84, 116, 42, 3, 211, 57, 186, 174, 208, 121, 253, 185, 210, 240];

const USIZE_BYTES: usize = (usize::BITS / 8) as usize;

#[inline]
pub fn base64_encode(bytes: impl AsRef<[u8]>) -> String {
    BASE64.encode(bytes)
}

#[inline]
pub fn base64_decode(string: impl AsRef<str>) -> std::result::Result<Vec<u8>, base64::DecodeError> {
    BASE64.decode(string.as_ref())
}

#[inline]
pub fn safe_random_u64() -> u64 {
    ChaCha20Rng::from_entropy().next_u64()
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    EllipticCurve(#[from] k256::elliptic_curve::Error),

    #[error(transparent)]
    Signature(#[from] k256::ecdsa::Error),

    #[error(transparent)]
    Base64(#[from] base64::DecodeError)
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PublicKey(k256::PublicKey);

impl PublicKey {
    pub fn to_bytes(&self) -> [u8; 33] {
        let bytes = self.0.to_sec1_bytes();

        // k256::PublicKey::to_sec1_bytes() should always return 33 bytes
        // but it's better to be safe
        assert_eq!(bytes.len(), 33);

        let mut result = [0_u8; 33];

        result.copy_from_slice(&bytes);

        result
    }

    pub fn from_bytes(bytes: impl AsRef<[u8]>) -> Result<Self, Error> {
        Ok(Self(k256::PublicKey::from_sec1_bytes(bytes.as_ref())?))
    }

    pub fn to_base64(&self) -> String {
        base64_encode(self.to_bytes())
    }

    pub fn from_base64(base64: impl AsRef<str>) -> Result<Self, Error> {
        Self::from_bytes(base64_decode(base64)?)
    }

    /// Verify signature of the message
    pub fn verify_signature(&self, message: impl AsRef<[u8]>, signature: impl AsRef<[u8]>) -> Result<bool, Error> {
        let signature = k256::ecdsa::Signature::from_slice(signature.as_ref())?;

        Ok(k256::ecdsa::VerifyingKey::from(&self.0)
            .verify(message.as_ref(), &signature)
            .is_ok())
    }

    /// Decode encapsulated signature, returning the original message if the signature is valid
    pub fn decode_encapsulated_signature(&self, capsule: impl AsRef<[u8]>) -> Result<Option<Vec<u8>>, Error> {
        let capsule = capsule.as_ref();

        let mut sign_len_bytes = [0_u8; USIZE_BYTES];

        sign_len_bytes.copy_from_slice(&capsule[..USIZE_BYTES]);

        let sign_len = usize::from_be_bytes(sign_len_bytes);

        let signature = &capsule[USIZE_BYTES..sign_len + USIZE_BYTES];
        let message = &capsule[sign_len + USIZE_BYTES..];

        if !self.verify_signature(message, signature)? {
            return Ok(None);
        }

        Ok(Some(message.to_vec()))
    }
}

impl std::hash::Hash for PublicKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.to_bytes().hash(state);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecretKey(k256::SecretKey);

impl SecretKey {
    pub fn random() -> Self {
        Self(k256::SecretKey::random(&mut ChaCha20Rng::from_entropy()))
    }

    pub fn random_from(rand: &mut impl CryptoRngCore) -> Self {
        Self(k256::SecretKey::random(rand))
    }

    pub fn public_key(&self) -> PublicKey {
        PublicKey(self.0.public_key())
    }

    pub fn serialize(&self) -> Vec<u8> {
        self.0.to_bytes().to_vec()
    }

    pub fn deserialize(bytes: impl AsRef<[u8]>) -> Result<Self, Error> {
        Ok(Self(k256::SecretKey::from_slice(bytes.as_ref())?))
    }

    pub fn to_base64(&self) -> String {
        base64_encode(self.serialize())
    }

    pub fn from_base64(base64: impl AsRef<str>) -> Result<Self, Error> {
        Self::deserialize(base64_decode(base64)?)
    }

    pub fn create_shared_secret(&self, public_key: &PublicKey, salt: Option<&[u8]>) -> [u8; 32] {
        let diffie_hellman = k256::ecdh::diffie_hellman(
            self.0.to_nonzero_scalar(),
            public_key.0.as_affine()
        );

        let generator = diffie_hellman.extract::<k256::sha2::Sha256>(salt);

        // sha2's block length is 32 bytes
        // so generator can do up to 8160 (32 * 255) bytes long secrets
        let mut secret = [0_u8; 32];

        unsafe {
            generator.expand(&HKDF_INFO, &mut secret)
                .unwrap_unchecked();
        }

        secret
    }

    /// Create a signature for the given message
    /// 
    /// To verify the signature you have to store the original message
    pub fn create_signature(&self, message: impl AsRef<[u8]>) -> Vec<u8> {
        let sign: k256::ecdsa::Signature = k256::ecdsa::SigningKey::from(&self.0)
            .sign(message.as_ref());

        sign.to_vec()
    }

    /// Create an encapsulated signature for the given message
    /// 
    /// The message will be stored in the signature
    pub fn create_encapsulated_signature(&self, message: impl AsRef<[u8]>) -> Vec<u8> {
        let message = message.as_ref();
        let sign = self.create_signature(message);

        let mut capsule = Vec::with_capacity(sign.len() + message.len() + USIZE_BYTES);

        capsule.extend(&sign.len().to_be_bytes());
        capsule.extend(sign);
        capsule.extend(message);

        capsule
    }
}

impl std::hash::Hash for SecretKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.serialize().hash(state);
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for SecretKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: serde::Serializer {
        use serde::ser::SerializeStruct;

        let mut sec = serializer.serialize_struct("SecretKey", 1)?;

        sec.serialize_field("0", &self.serialize())?;

        sec.end()
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for SecretKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: serde::Deserializer<'de> {
        use serde::de::Error;

        struct Visitor;

        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = Vec<u8>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(formatter, "bytes sequence expected")
            }

            fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
            where E: serde::de::Error {
                Ok(v.to_vec())
            }
        }

        let secret_key = deserializer.deserialize_struct("SecretKey", &["0"], Visitor)?;

        let secret_key = SecretKey::deserialize(secret_key)
            .map_err(D::Error::custom)?;

        Ok(secret_key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize() {
        let secret = SecretKey::random();
        let public = secret.public_key();

        assert_eq!(SecretKey::deserialize(secret.serialize()).unwrap(), secret);
        assert_eq!(PublicKey::from_bytes(public.to_bytes()).unwrap(), public);
    }

    #[test]
    fn base64() {
        let secret = SecretKey::random();
        let public = secret.public_key();

        assert_eq!(SecretKey::from_base64(secret.to_base64()).unwrap(), secret);
        assert_eq!(PublicKey::from_base64(public.to_base64()).unwrap(), public);
    }

    #[test]
    fn shared_secret() {
        let secret_1 = SecretKey::random();
        let secret_2 = SecretKey::random();

        let key_1 = secret_1.create_shared_secret(&secret_2.public_key(), None);
        let key_2 = secret_2.create_shared_secret(&secret_1.public_key(), None);

        assert!(key_1 == key_2);
    }

    #[test]
    fn signature() {
        let secret = SecretKey::random();

        let message = b"Hello, World!";
        let signature = secret.create_signature(message);

        assert!(secret.public_key().verify_signature(message, signature).unwrap());
    }

    #[test]
    fn encapsulated_signature() {
        let secret = SecretKey::random();

        let signature = secret.create_encapsulated_signature(b"Hello, World!");

        assert_eq!(secret.public_key().decode_encapsulated_signature(signature).unwrap(), Some(b"Hello, World!".to_vec()));
    }
}
