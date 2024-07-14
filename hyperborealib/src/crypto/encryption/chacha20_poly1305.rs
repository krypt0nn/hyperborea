use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce, KeyInit, Error};
use chacha20poly1305::aead::Aead;

/// Standard nonce value used by the hyperborea protocol
/// for ChaCha20-Poly1305 encryption algorithm.
pub const NONCE: [u8; 32] = [
    197, 93,  164, 33,  103, 79,  137, 143,
    186, 253, 74,  220, 202, 54,  182, 47,
    236, 124, 175, 177, 150, 127, 188, 104,
    132, 227, 10,  55,  6,   15,  106, 194
];

/// Encrypt given value.
/// 
/// This function will automatically apply standard nonce.
pub fn encrypt(data: impl AsRef<[u8]>, secret: &[u8; 32]) -> Result<Vec<u8>, Error> {
    let key = Key::from_slice(secret);

    ChaCha20Poly1305::new(key)
        .encrypt(Nonce::from_slice(&NONCE), data.as_ref())
}

/// Decrypt given value.
/// 
/// This function will automatically apply standard nonce.
pub fn decrypt(data: impl AsRef<[u8]>, secret: &[u8; 32]) -> Result<Vec<u8>, Error> {
    let key = Key::from_slice(secret);

    ChaCha20Poly1305::new(key)
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
