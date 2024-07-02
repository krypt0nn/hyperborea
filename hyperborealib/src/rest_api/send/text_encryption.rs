use super::Error;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TextEncryption {
    #[default]
    None,

    Aes256Gcm,
    ChaCha20Poly1305
}

// TODO: consider moving these algorithms under crate::crypto

impl TextEncryption {
    pub fn encrypt(&self, data: impl AsRef<[u8]>, secret: [u8; 32]) -> Result<Vec<u8>, Error> {
        match self {
            Self::None => Ok(data.as_ref().to_vec()),

            Self::Aes256Gcm => {
                use aes_gcm::{Aes256Gcm, Nonce, KeyInit};
                use aes_gcm::aead::Aead;

                let aes = Aes256Gcm::new_from_slice(&secret)
                    .map_err(|err| Error::TextEncryptionFailed(err.into()))?;

                let encrypted = aes.encrypt(&Nonce::default(), data.as_ref())
                    .map_err(|err| Error::TextEncryptionFailed(err.into()))?;

                Ok(encrypted)
            }

            Self::ChaCha20Poly1305 => {
                use chacha20poly1305::{ChaCha20Poly1305, Nonce, KeyInit};
                use chacha20poly1305::aead::Aead;

                let chacha = ChaCha20Poly1305::new_from_slice(&secret)
                    .map_err(|err| Error::TextEncryptionFailed(err.into()))?;

                let encrypted = chacha.encrypt(&Nonce::default(), data.as_ref())
                    .map_err(|err| Error::TextEncryptionFailed(err.into()))?;

                Ok(encrypted)
            }
        }
    }

    pub fn decrypt(&self, data: impl AsRef<[u8]>, secret: [u8; 32]) -> Result<Vec<u8>, Error> {
        match self {
            Self::None => Ok(data.as_ref().to_vec()),

            Self::Aes256Gcm => {
                use aes_gcm::{Aes256Gcm, Nonce, KeyInit};
                use aes_gcm::aead::Aead;

                let aes = Aes256Gcm::new_from_slice(&secret)
                    .map_err(|err| Error::TextDecryptionFailed(err.into()))?;

                let decrypted = aes.decrypt(&Nonce::default(), data.as_ref())
                    .map_err(|err| Error::TextDecryptionFailed(err.into()))?;

                Ok(decrypted)
            }

            Self::ChaCha20Poly1305 => {
                use chacha20poly1305::{ChaCha20Poly1305, Nonce, KeyInit};
                use chacha20poly1305::aead::Aead;

                let chacha = ChaCha20Poly1305::new_from_slice(&secret)
                    .map_err(|err| Error::TextDecryptionFailed(err.into()))?;

                let decrypted = chacha.decrypt(&Nonce::default(), data.as_ref())
                    .map_err(|err| Error::TextDecryptionFailed(err.into()))?;

                Ok(decrypted)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encryption_decryption() -> Result<(), Error> {
        let encryptions = [
            TextEncryption::None,
            TextEncryption::Aes256Gcm,
            TextEncryption::ChaCha20Poly1305
        ];

        let key = b"amogus aboba banana aboba amogus";

        for encryption in encryptions {
            let encrypted = encryption.encrypt(b"Hello, World!", *key)?;
            let decrypted = encryption.decrypt(encrypted, *key)?;

            assert_eq!(decrypted, b"Hello, World!");
        }

        Ok(())
    }
}
