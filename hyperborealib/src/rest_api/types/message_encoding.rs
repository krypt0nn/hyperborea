use std::str::FromStr;

use crate::crypto::prelude::*;

use super::MessagesError;

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
    pub fn forward(&self, message: impl AsRef<[u8]>, secret: &[u8; 32], level: CompressionLevel) -> Result<String, MessagesError> {
        let message = self.compression.compress(message, level)?;
        let message = self.encryption.encrypt(message, secret)?;

        Ok(self.encoding.encode(message))
    }

    /// Cease compression, encryption and encoding
    /// from the given data.
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
