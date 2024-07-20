use std::str::FromStr;

use crate::crypto::prelude::*;

use super::MessagesError;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Encoding of the message.
/// 
/// This is a standard type declared in the
/// hyperborea protocol's paper.
pub struct MessageEncoding {
    pub encoding: Encoding,
    pub encryption: Encryption,
    pub compression: Compression
}

impl MessageEncoding {
    #[inline]
    /// Create new message encoding.
    /// 
    /// - `encoding` must contain algorithm which will
    ///   convert raw bytes data into UTF-8 compatible text.
    /// 
    /// - `encryption` must contain algorithm which will
    ///   encrypt raw bytes using secret key.
    /// 
    /// - `compression` must contain algorithm which will
    ///   compress raw bytes to smaller size to reduce
    ///   network transportation bandwith.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use hyperborealib::crypto::prelude::*;
    /// use hyperborealib::rest_api::prelude::*;
    /// 
    /// let encoding = MessageEncoding::new(
    ///     Encoding::Base64,
    ///     Encryption::None,
    ///     Compression::Brotli
    /// );
    /// ```
    pub fn new(encoding: Encoding, encryption: Encryption, compression: Compression) -> Self {
        Self {
            encoding,
            encryption,
            compression
        }
    }

    /// Apply compression, encryption and encoding
    /// to the given data.
    /// 
    /// - `message` must be a data slice you need to process.
    /// 
    /// - `secret` must be a secret key used for encryption.
    /// 
    /// - `level` must be a data compression level.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use std::str::FromStr;
    /// 
    /// use hyperborealib::crypto::prelude::*;
    /// use hyperborealib::rest_api::prelude::*;
    /// 
    /// let encoding = MessageEncoding::from_str("base64/chacha20-poly1305/brotli").unwrap();
    /// 
    /// let processed = encoding.forward(
    ///     b"Hello, World!",
    ///     b"example 32 bytes long key ......",
    ///     CompressionLevel::default()
    /// ).unwrap();
    /// 
    /// assert_eq!(processed, "6aHXWENbkDrFuBQxQIa5RiPGgQ1_Je2rVeYw7Zt19VB8");
    /// ```
    pub fn forward(&self, message: impl AsRef<[u8]>, secret: &[u8; 32], level: CompressionLevel) -> Result<String, MessagesError> {
        let message = self.compression.compress(message, level)?;
        let message = self.encryption.encrypt(message, secret)?;

        Ok(self.encoding.encode(message))
    }

    /// Cease compression, encryption and encoding
    /// from the given data.
    /// 
    /// - `message` must be a data slice you need to process.
    /// 
    /// - `secret` must be a secret key used for decryption.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use std::str::FromStr;
    /// 
    /// use hyperborealib::crypto::prelude::*;
    /// use hyperborealib::rest_api::prelude::*;
    /// 
    /// let encoding = MessageEncoding::from_str("base64/chacha20-poly1305/brotli").unwrap();
    /// 
    /// let processed = encoding.backward(
    ///     "6aHXWENbkDrFuBQxQIa5RiPGgQ1_Je2rVeYw7Zt19VB8",
    ///     b"example 32 bytes long key ......"
    /// ).unwrap();
    /// 
    /// assert_eq!(processed, b"Hello, World!");
    /// ```
    pub fn backward(&self, message: impl AsRef<str>, secret: &[u8; 32]) -> Result<Vec<u8>, MessagesError> {
        let message = self.encoding.decode(message)?;
        let message = self.encryption.decrypt(message, secret)?;

        Ok(self.compression.decompress(message)?)
    }
}

impl FromStr for MessageEncoding {
    type Err = MessagesError;

    fn from_str(str: &str) -> Result<Self, Self::Err> {
        let parts = str.split('/')
            .collect::<Vec<_>>();

        match parts.len() {
            // <encoding>
            1 => {
                Ok(Self {
                    encoding: Encoding::from_str(parts[0])?,
                    encryption: Encryption::None,
                    compression: Compression::None
                })
            }

            // <encoding>/<encryption>
            // <encoding>/<compression>
            2 => {
                let encoding = Encoding::from_str(parts[0])?;

                if let Ok(encryption) = Encryption::from_str(parts[1]) {
                    Ok(Self {
                        encoding,
                        encryption,
                        compression: Compression::None
                    })
                }

                else {
                    Ok(Self {
                        encoding,
                        encryption: Encryption::None,
                        compression: Compression::from_str(parts[1])?
                    })
                }
            }

            // <encoding>/<encryption>/<compression>
            3 => {
                Ok(Self {
                    encoding: Encoding::from_str(parts[0])?,
                    encryption: Encryption::from_str(parts[1])?,
                    compression: Compression::from_str(parts[2])?
                })
            }

            _ => Err(MessagesError::WrongMessageEncodingFormat(str.to_string()))
        }
    }
}

impl std::fmt::Display for MessageEncoding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match (self.encoding, self.encryption, self.compression) {
            (encoding, Encryption::None, Compression::None) => write!(f, "{encoding}"),
            (encoding, encryption, Compression::None) => write!(f, "{encoding}/{encryption}"),
            (encoding, Encryption::None, compression) => write!(f, "{encoding}/{compression}"),
            (encoding, encryption, compression) => write!(f, "{encoding}/{encryption}/{compression}")
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    pub fn get_encodings() -> Result<Vec<MessageEncoding>, MessagesError> {
        Ok(vec![
            MessageEncoding::from_str("base64")?,

            MessageEncoding::from_str("base64/deflate")?,
            MessageEncoding::from_str("base64/brotli")?,

            MessageEncoding::from_str("base64/aes256-gcm")?,
            MessageEncoding::from_str("base64/chacha20-poly1305")?,

            MessageEncoding::from_str("base64/aes256-gcm/deflate")?,
            MessageEncoding::from_str("base64/chacha20-poly1305/deflate")?,
            MessageEncoding::from_str("base64/aes256-gcm/brotli")?,
            MessageEncoding::from_str("base64/chacha20-poly1305/brotli")?
        ])
    }

    #[test]
    fn parse() -> Result<(), MessagesError> {
        assert!(MessageEncoding::from_str("aboba").is_err());

        for encoding in get_encodings()? {
            assert_eq!(MessageEncoding::from_str(&encoding.to_string())?, encoding);
        }

        Ok(())
    }
}
