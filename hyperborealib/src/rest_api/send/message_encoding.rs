use crate::crypto::prelude::*;

use super::Error;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MessageEncoding {
    pub encoding: Encoding,
    pub encryption: Encryption,
    pub compression: Compression
}

impl MessageEncoding {
    #[inline]
    pub fn new(encoding: Encoding, encryption: Encryption, compression: Compression) -> Self {
        Self {
            encoding,
            encryption,
            compression
        }
    }

    /// Apply compression, encryption and encoding
    /// to the given data.
    pub fn forward(&self, message: impl AsRef<[u8]>, secret: &[u8; 32]) -> Result<String, Error> {
        let message = self.compression.compress(message)?;
        let message = self.encryption.encrypt(message, secret)?;

        Ok(self.encoding.encode(message))
    }

    /// Cease compression, encryption and encoding
    /// from the given data.
    pub fn backward(&self, message: impl AsRef<str>, secret: &[u8; 32]) -> Result<Vec<u8>, Error> {
        let message = self.encoding.decode(message)?;
        let message = self.encryption.decrypt(message, secret)?;

        Ok(self.compression.decompress(message)?)
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(str: impl AsRef<str>) -> Result<Self, Error> {
        match str.as_ref() {
            "base64/plain" => Ok(Self {
                encoding: Encoding::Base64,
                encryption: Encryption::None,
                compression: Compression::None
            }),

            "base64/deflate" => Ok(Self {
                encoding: Encoding::Base64,
                encryption: Encryption::None,
                compression: Compression::Deflate
            }),

            "base64/aes256-gcm" => Ok(Self {
                encoding: Encoding::Base64,
                encryption: Encryption::Aes256Gcm,
                compression: Compression::None
            }),

            "base64/chacha20-poly1305" => Ok(Self {
                encoding: Encoding::Base64,
                encryption: Encryption::ChaCha20Poly1305,
                compression: Compression::None
            }),

            "base64/aes256-gcm/deflate" => Ok(Self {
                encoding: Encoding::Base64,
                encryption: Encryption::Aes256Gcm,
                compression: Compression::Deflate
            }),

            "base64/chacha20-poly1305/deflate" => Ok(Self {
                encoding: Encoding::Base64,
                encryption: Encryption::ChaCha20Poly1305,
                compression: Compression::Deflate
            }),

            str => Err(Error::WrongMessageEncodingFormat(str.to_string()))
        }
    }

    pub fn to_str(&self) -> &'static str {
        match self.encryption {
            Encryption::None => match self.compression {
                Compression::None    => "base64/plain",
                Compression::Deflate => "base64/deflate"
            },

            Encryption::Aes256Gcm => match self.compression {
                Compression::None    => "base64/aes256-gcm",
                Compression::Deflate => "base64/aes256-gcm/deflate"
            },

            Encryption::ChaCha20Poly1305 => match self.compression {
                Compression::None    => "base64/chacha20-poly1305",
                Compression::Deflate => "base64/chacha20-poly1305/deflate"
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize() -> Result<(), Error> {
        let encodings = [
            MessageEncoding::from_str("base64/plain")?,
            MessageEncoding::from_str("base64/deflate")?,
            MessageEncoding::from_str("base64/aes256-gcm")?,
            MessageEncoding::from_str("base64/chacha20-poly1305")?,
            MessageEncoding::from_str("base64/aes256-gcm/deflate")?,
            MessageEncoding::from_str("base64/chacha20-poly1305/deflate")?
        ];

        match MessageEncoding::from_str("aboba") {
            Err(Error::WrongMessageEncodingFormat(str)) if str == "aboba" => (),

            _ => panic!("Message encoding expected to fail at value 'aboba'")
        }

        for encoding in encodings {
            assert_eq!(MessageEncoding::from_str(encoding.to_str())?, encoding);
        }

        Ok(())
    }
}
