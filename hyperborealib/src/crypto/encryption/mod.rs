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
/// General data encryption implementation.
/// 
/// ```rust
/// use std::str::FromStr;
/// 
/// use hyperborealib::crypto::encryption::Encryption;
/// 
/// assert_eq!(Encryption::None.to_string(),             "plain");
/// assert_eq!(Encryption::Aes256Gcm.to_string(),        "aes256-gcm");
/// assert_eq!(Encryption::ChaCha20Poly1305.to_string(), "chacha20-poly1305");
/// 
/// assert_eq!(Encryption::from_str("none").unwrap(),              Encryption::None);
/// assert_eq!(Encryption::from_str("plain").unwrap(),             Encryption::None);
/// assert_eq!(Encryption::from_str("aes256-gcm").unwrap(),        Encryption::Aes256Gcm);
/// assert_eq!(Encryption::from_str("chacha20-poly1305").unwrap(), Encryption::ChaCha20Poly1305);
/// 
/// assert_eq!("none".parse::<Encryption>().unwrap(),              Encryption::None);
/// assert_eq!("plain".parse::<Encryption>().unwrap(),             Encryption::None);
/// assert_eq!("aes256-gcm".parse::<Encryption>().unwrap(),        Encryption::Aes256Gcm);
/// assert_eq!("chacha20-poly1305".parse::<Encryption>().unwrap(), Encryption::ChaCha20Poly1305);
/// ```
pub enum Encryption {
    #[default]
    None,

    Aes256Gcm,
    ChaCha20Poly1305
}

impl Encryption {
    /// Encrypt given data using selected encryption algorithm.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use hyperborealib::crypto::encryption::Encryption;
    /// 
    /// let original = b"Hello, World!";
    /// let secret   = b"32 bytes long secret key .......";
    /// 
    /// let aes    = Encryption::Aes256Gcm.encrypt(original, &secret).unwrap();
    /// let chacha = Encryption::ChaCha20Poly1305.encrypt(original, &secret).unwrap();
    /// 
    /// assert_ne!(aes, original);
    /// assert_ne!(chacha, original);
    /// ```
    pub fn encrypt(&self, data: impl AsRef<[u8]>, secret: &[u8; 32]) -> Result<Vec<u8>, Error> {
        match self {
            Self::None => Ok(data.as_ref().to_vec()),

            Self::Aes256Gcm => aes256_gcm::encrypt(data, secret)
                .map_err(|err| Error::Encryption(err.into())),

            Self::ChaCha20Poly1305 => chacha20_poly1305::encrypt(data, secret)
                .map_err(|err| Error::Encryption(err.into()))
        }
    }

    /// Encrypt given data using selected encryption algorithm.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use hyperborealib::crypto::encryption::Encryption;
    /// 
    /// let original = b"Hello, World!";
    /// let secret   = b"32 bytes long secret key .......";
    /// 
    /// let aes    = Encryption::Aes256Gcm.encrypt(original, &secret).unwrap();
    /// let chacha = Encryption::ChaCha20Poly1305.encrypt(original, &secret).unwrap();
    /// 
    /// let aes    = Encryption::Aes256Gcm.decrypt(aes, &secret).unwrap();
    /// let chacha = Encryption::ChaCha20Poly1305.decrypt(chacha, &secret).unwrap();
    /// 
    /// assert_eq!(aes, original);
    /// assert_eq!(chacha, original);
    /// ```
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

impl std::str::FromStr for Encryption {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "none" | "plain" => Ok(Self::None),

            "aes256-gcm" => Ok(Self::Aes256Gcm),
            "chacha20-poly1305"  => Ok(Self::ChaCha20Poly1305),

            _ => Err(Error::UnknownEncryption(value.to_string()))
        }
    }
}

impl std::fmt::Display for Encryption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None             => write!(f, "plain"),
            Self::Aes256Gcm        => write!(f, "aes256-gcm"),
            Self::ChaCha20Poly1305 => write!(f, "chacha20-poly1305")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn encryptions() -> &'static [(Encryption, &'static str)] {
        &[
            (Encryption::None,             "plain"),
            (Encryption::Aes256Gcm,        "aes256-gcm"),
            (Encryption::ChaCha20Poly1305, "chacha20-poly1305")
        ]
    }

    #[test]
    fn encrypt_decrypt() -> Result<(), Error> {
        let key = b"amogus aboba banana aboba amogus";

        for (encryption, _) in encryptions() {
            let encrypted = encryption.encrypt(b"Hello, World!", key)?;
            let decrypted = encryption.decrypt(encrypted, key)?;

            assert_eq!(decrypted, b"Hello, World!");
        }

        Ok(())
    }

    #[test]
    fn display() {
        for (encryption, name) in encryptions() {
            assert_eq!(encryption.to_string(), *name);
        }
    }

    #[test]
    fn parse() -> Result<(), Error> {
        for (encryption, name) in encryptions() {
            assert_eq!(name.parse::<Encryption>()?, *encryption);
        }

        Ok(())
    }
}
