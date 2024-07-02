use crate::crypto::{base64_encode, base64_decode};

use super::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TextEncoding {
    Base64
}

impl TextEncoding {
    pub fn encode(&self, data: impl AsRef<[u8]>) -> String {
        match self {
            Self::Base64 => base64_encode(data)
        }
    }

    pub fn decode(&self, text: impl AsRef<str>) -> Result<Vec<u8>, Error> {
        match self {
            Self::Base64 => base64_decode(text)
                .map_err(|err| Error::TextDecodingError(err.into()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_decode() -> Result<(), Error> {
        let encodings = [
            TextEncoding::Base64
        ];

        for encoding in encodings {
            let encoded = encoding.encode(b"Hello, World!");
            let decoded = encoding.decode(encoded)?;

            assert_eq!(decoded, b"Hello, World!");
        }

        Ok(())
    }
}
