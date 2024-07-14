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
    /// Encode given binary data using selected text encoding.
    /// 
    /// ```rust
    /// use hyperborealib::crypto::encoding::Encoding;
    /// 
    /// assert_eq!(Encoding::Base64.encode(b"Hello, World!"), "SGVsbG8sIFdvcmxkIQ==");
    /// ```
    pub fn encode(&self, data: impl AsRef<[u8]>) -> String {
        match self {
            Self::Base64 => base64::encode(data)
        }
    }

    /// Decode given text into a binary data using selected encoding.
    /// 
    /// ```rust
    /// use hyperborealib::crypto::encoding::Encoding;
    /// 
    /// assert_eq!(Encoding::Base64.decode("SGVsbG8sIFdvcmxkIQ==").unwrap(), b"Hello, World!");
    /// ```
    pub fn decode(&self, text: impl AsRef<str>) -> Result<Vec<u8>, Error> {
        match self {
            Self::Base64 => base64::decode(text)
                .map_err(|err| Error::Decoding(err.into()))
        }
    }
}

impl std::str::FromStr for Encoding {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "base64" => Ok(Self::Base64),

            _ => Err(Error::UnknownEncoding(value.to_string()))
        }
    }
}

impl std::fmt::Display for Encoding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Base64 => write!(f, "base64")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn encodings() -> &'static [(Encoding, &'static str)] {
        &[
            (Encoding::Base64, "base64")
        ]
    }

    #[test]
    fn encode_decode() -> Result<(), Error> {
        for (encoding, _) in encodings() {
            let encoded = encoding.encode(b"Hello, World!");
            let decoded = encoding.decode(encoded)?;

            assert_eq!(decoded, b"Hello, World!");
        }

        Ok(())
    }

    #[test]
    fn display() {
        for (encoding, name) in encodings() {
            assert_eq!(encoding.to_string(), *name);
        }
    }

    #[test]
    fn parse() -> Result<(), Error> {
        for (encoding, name) in encodings() {
            assert_eq!(name.parse::<Encoding>()?, *encoding);
        }

        Ok(())
    }
}
