use super::Error;

pub mod aes256_gcm;
pub mod chacha20_poly1305;

pub mod prelude {
    pub use super::Encryption;

    pub use super::aes256_gcm::{
        encrypt as aes256_gcm_encrypt,
        decrypt as aes256_gcm_decrypt
    };

    pub use super::chacha20_poly1305::{
        encrypt as chacha20_poly1305_encrypt,
        decrypt as chacha20_poly1305_decrypt
    };
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Encryption {
    #[default]
    None,

    Aes256Gcm,
    ChaCha20Poly1305
}

impl Encryption {
    pub fn encrypt(&self, data: impl AsRef<[u8]>, secret: &[u8; 32]) -> Result<Vec<u8>, Error> {
        match self {
            Self::None => Ok(data.as_ref().to_vec()),

            Self::Aes256Gcm => aes256_gcm::encrypt(data, secret)
                .map_err(|err| Error::Encryption(err.into())),

            Self::ChaCha20Poly1305 => chacha20_poly1305::encrypt(data, secret)
                .map_err(|err| Error::Encryption(err.into()))
        }
    }

    pub fn decrypt(&self, data: impl AsRef<[u8]>, secret: &[u8; 32]) -> Result<Vec<u8>, Error> {
        match self {
            Self::None => Ok(data.as_ref().to_vec()),

            Self::Aes256Gcm => aes256_gcm::decrypt(data, secret)
                .map_err(|err| Error::Decryption(err.into())),

            Self::ChaCha20Poly1305 => chacha20_poly1305::decrypt(data, secret)
                .map_err(|err| Error::Decryption(err.into()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encryption_decryption() -> Result<(), Error> {
        let encryptions = [
            Encryption::None,
            Encryption::Aes256Gcm,
            Encryption::ChaCha20Poly1305
        ];

        let key = b"amogus aboba banana aboba amogus";

        for encryption in encryptions {
            let encrypted = encryption.encrypt(b"Hello, World!", key)?;
            let decrypted = encryption.decrypt(encrypted, key)?;

            assert_eq!(decrypted, b"Hello, World!");
        }

        Ok(())
    }
}
