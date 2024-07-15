use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce, KeyInit, Error};
use chacha20poly1305::aead::Aead;

/// Standard nonce value used by the hyperborea protocol
/// for ChaCha20-Poly1305 encryption algorithm.
/// 
/// Randomly generated using [random.org](https://www.random.org/sequences/?min=0&max=255&col=12&format=html&rnd=new) service.
pub const NONCE: [u8; 12] = [197, 93, 164, 33, 103, 79, 137, 143, 186, 253, 74, 220];

/// Encrypt given value.
/// 
/// This function will automatically apply standard nonce.
/// 
/// # Example
/// 
/// ```rust
/// use hyperborealib::crypto::encryption::chacha20_poly1305;
/// 
/// let original = b"Hello, World!";
/// let secret   = b"32 bytes long secret key .......";
/// 
/// let encrypted = chacha20_poly1305::encrypt(original, &secret).unwrap();
/// 
/// assert_ne!(encrypted, original);
/// ```
pub fn encrypt(data: impl AsRef<[u8]>, secret: &[u8; 32]) -> Result<Vec<u8>, Error> {
    let key = Key::from_slice(secret);

    ChaCha20Poly1305::new(key)
        .encrypt(Nonce::from_slice(&NONCE), data.as_ref())
}

/// Decrypt given value.
/// 
/// This function will automatically apply standard nonce.
/// 
/// # Example
/// 
/// ```rust
/// use hyperborealib::crypto::encryption::chacha20_poly1305;
/// 
/// let original = b"Hello, World!";
/// let secret   = b"32 bytes long secret key .......";
/// 
/// let encrypted = chacha20_poly1305::encrypt(original, &secret).unwrap();
/// let decrypted = chacha20_poly1305::decrypt(&encrypted, &secret).unwrap();
/// 
/// assert_ne!(encrypted, original);
/// assert_eq!(decrypted, original);
/// ```
pub fn decrypt(data: impl AsRef<[u8]>, secret: &[u8; 32]) -> Result<Vec<u8>, Error> {
    let key = Key::from_slice(secret);

    ChaCha20Poly1305::new(key)
        .decrypt(Nonce::from_slice(&NONCE), data.as_ref())
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    pub fn encrypt_decrypt() -> Result<(), Error> {
        let key = b"amogus aboba banana aboba amogus";

        assert_eq!(decrypt(encrypt(b"Hello, World!", key)?, key)?, b"Hello, World!");

        Ok(())
    }
}
