use super::Error;

pub mod base64;

pub mod prelude {
    pub use super::Encoding;

    pub use super::base64::{
        encode as base64_encode,
        decode as base64_decode
    };
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Encoding {
    #[default]
    Base64
}

impl Encoding {
    pub fn encode(&self, data: impl AsRef<[u8]>) -> String {
        match self {
            Self::Base64 => base64::encode(data)
        }
    }

    pub fn decode(&self, text: impl AsRef<str>) -> Result<Vec<u8>, Error> {
        match self {
            Self::Base64 => base64::decode(text)
                .map_err(|err| Error::Decoding(err.into()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_decode() -> Result<(), Error> {
        let encodings = [
            Encoding::Base64
        ];

        for encoding in encodings {
            let encoded = encoding.encode(b"Hello, World!");
            let decoded = encoding.decode(encoded)?;

            assert_eq!(decoded, b"Hello, World!");
        }

        Ok(())
    }
}
