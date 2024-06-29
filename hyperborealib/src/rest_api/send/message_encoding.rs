use super::{
    TextEncoding,
    TextEncryption,
    TextCompression,
    Error
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MessageEncoding {
    pub encoding: TextEncoding,
    pub encryption: TextEncryption,
    pub compression: TextCompression
}

impl MessageEncoding {
    #[inline]
    pub fn new(encoding: TextEncoding, encryption: TextEncryption, compression: TextCompression) -> Self {
        Self {
            encoding,
            encryption,
            compression
        }
    }

    /// Apply compression, encryption and encoding
    /// to the given data.
    pub fn forward(&self, message: impl AsRef<[u8]>, secret: [u8; 32]) -> Result<String, Error> {
        let message = self.compression.compress(message)?;
        let message = self.encryption.encrypt(message, secret)?;

        Ok(self.encoding.encode(message))
    }

    /// Cease compression, encryption and encoding
    /// from the given data.
    pub fn backward(&self, message: impl AsRef<str>, secret: [u8; 32]) -> Result<Vec<u8>, Error> {
        let message = self.encoding.decode(message)?;
        let message = self.encryption.decrypt(message, secret)?;

        self.compression.decompress(message)
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(str: impl AsRef<str>) -> Result<Self, Error> {
        match str.as_ref() {
            "base64/plain" => Ok(Self {
                encoding: TextEncoding::Base64,
                encryption: TextEncryption::None,
                compression: TextCompression::None
            }),

            "base64/deflate" => Ok(Self {
                encoding: TextEncoding::Base64,
                encryption: TextEncryption::None,
                compression: TextCompression::Deflate
            }),

            "base64/aes256-gcm" => Ok(Self {
                encoding: TextEncoding::Base64,
                encryption: TextEncryption::Aes256Gcm,
                compression: TextCompression::None
            }),

            "base64/chacha20-poly1305" => Ok(Self {
                encoding: TextEncoding::Base64,
                encryption: TextEncryption::ChaCha20Poly1305,
                compression: TextCompression::None
            }),

            "base64/aes256-gcm/deflate" => Ok(Self {
                encoding: TextEncoding::Base64,
                encryption: TextEncryption::Aes256Gcm,
                compression: TextCompression::Deflate
            }),

            "base64/chacha20-poly1305/deflate" => Ok(Self {
                encoding: TextEncoding::Base64,
                encryption: TextEncryption::ChaCha20Poly1305,
                compression: TextCompression::Deflate
            }),

            str => Err(Error::WrongMessageEncodingFormat(str.to_string()))
        }
    }

    pub fn to_str(&self) -> &'static str {
        match self.encryption {
            TextEncryption::None => match self.compression {
                TextCompression::None    => "base64/plain",
                TextCompression::Deflate => "base64/deflate"
            },

            TextEncryption::Aes256Gcm => match self.compression {
                TextCompression::None    => "base64/aes256-gcm",
                TextCompression::Deflate => "base64/aes256-gcm/deflate"
            },

            TextEncryption::ChaCha20Poly1305 => match self.compression {
                TextCompression::None    => "base64/chacha20-poly1305",
                TextCompression::Deflate => "base64/chacha20-poly1305/deflate"
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
