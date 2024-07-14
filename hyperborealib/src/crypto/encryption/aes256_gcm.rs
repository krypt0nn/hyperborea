use aes_gcm::{Aes256Gcm, Key, Nonce, KeyInit, Error};
use aes_gcm::aead::Aead;

/// Standard nonce value used by the hyperborea protocol
/// for AES256-GCM encryption algorithm.
/// 
/// Randomly generated using [random.org](https://www.random.org/sequences/?min=0&max=255&col=12&format=html&rnd=new) service.
pub const NONCE: [u8; 12] = [234, 90, 0, 39, 141, 73, 94, 100, 178, 97, 128, 118];

/// Encrypt given value.
/// 
/// This function will automatically apply standard nonce.
pub fn encrypt(data: impl AsRef<[u8]>, secret: &[u8; 32]) -> Result<Vec<u8>, Error> {
    let key = Key::<Aes256Gcm>::from_slice(secret);

    Aes256Gcm::new(key)
        .encrypt(Nonce::from_slice(&NONCE), data.as_ref())
}

/// Decrypt given value.
/// 
/// This function will automatically apply standard nonce.
pub fn decrypt(data: impl AsRef<[u8]>, secret: &[u8; 32]) -> Result<Vec<u8>, Error> {
    let key = Key::<Aes256Gcm>::from_slice(secret);

    Aes256Gcm::new(key)
        .decrypt(Nonce::from_slice(&NONCE), data.as_ref())
}

#[cfg(test)]
pub mod tests {
    use super::*;

    pub fn encrypt_decrypt() -> Result<(), Error> {
        let key = b"amogus aboba banana aboba amogus";

        assert_eq!(decrypt(encrypt(b"Hello, World!", key)?, key)?, b"Hello, World!");

        Ok(())
    }
}
