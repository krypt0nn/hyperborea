use rand_chacha::ChaCha20Rng;
use rand_chacha::rand_core::{CryptoRngCore, SeedableRng};

use k256::ecdsa::signature::Signer;

use crate::crypto::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecretKey(pub(crate) k256::SecretKey);

impl SecretKey {
    /// Generate new random secret key.
    pub fn random() -> Self {
        Self(k256::SecretKey::random(&mut ChaCha20Rng::from_entropy()))
    }

    /// Generate new random secret key from given
    /// random numbers generator backend.
    pub fn random_from(rand: &mut impl CryptoRngCore) -> Self {
        Self(k256::SecretKey::random(rand))
    }

    /// Create new public key from the current secrey key.
    pub fn public_key(&self) -> PublicKey {
        PublicKey::from(self.0.public_key())
    }

    /// Serialize secret key into a bytes vector.
    pub fn serialize(&self) -> Vec<u8> {
        self.0.to_bytes().to_vec()
    }

    /// Deserialize secret key from the bytes slice.
    pub fn deserialize(bytes: impl AsRef<[u8]>) -> Result<Self, CryptographyError> {
        Ok(Self(k256::SecretKey::from_slice(bytes.as_ref())?))
    }

    /// Serialize secret key into bytes vector and encode it
    /// into base 64 number.
    pub fn to_base64(&self) -> String {
        base64_encode(self.serialize())
    }

    /// Decode given base 64 number and deserialize
    /// a secret key from it.
    pub fn from_base64(base64: impl AsRef<str>) -> Result<Self, CryptographyError> {
        Self::deserialize(base64_decode(base64)
            .map_err(|err| CryptographyError::Decoding(err.into()))?)
    }

    /// Create shared secret key with a client with given public key.
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

    /// Create a signature for the given message.
    /// 
    /// To verify the signature you have to store the original message.
    pub fn create_signature(&self, message: impl AsRef<[u8]>) -> Vec<u8> {
        let sign: k256::ecdsa::Signature = k256::ecdsa::SigningKey::from(&self.0)
            .sign(message.as_ref());

        sign.to_vec()
    }

    /// Create an encapsulated signature for the given message.
    /// 
    /// The message will be stored in the signature.
    pub fn create_encapsulated_signature(&self, message: impl AsRef<[u8]>) -> Vec<u8> {
        let message = message.as_ref();
        let sign = self.create_signature(message);

        let mut capsule = Vec::with_capacity(sign.len() + message.len() + 8);

        capsule.extend(&(sign.len() as u64).to_be_bytes());
        capsule.extend(sign);
        capsule.extend(message);

        capsule
    }
}

impl From<k256::SecretKey> for SecretKey {
    #[inline]
    fn from(value: k256::SecretKey) -> Self {
        Self(value)
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
