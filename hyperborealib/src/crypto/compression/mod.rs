use super::Error;

pub mod deflate;
pub mod brotli;

pub mod prelude {
    pub use super::{
        Compression,
        CompressionLevel
    };

    pub use super::deflate::{
        compress as deflate_compress,
        decompress as deflate_decompress
    };

    pub use super::brotli::{
        compress as brotli_compress,
        decompress as brotli_decompress
    };
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CompressionLevel {
    /// Fastest possible compression speed.
    /// 
    /// Generally provides bad results, but helpful
    /// when you send many messages.
    Fast,

    #[default]
    /// Balanced compression speed.
    /// 
    /// Provides good results for affordable time and resources.
    Balanced,

    /// Best possible compression.
    /// 
    /// Will require many time and computation resources.
    Quality
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Compression {
    #[default]
    None,

    Deflate,
    Brotli
}

impl Compression {
    /// Compress given data with selected compression algorithm.
    /// 
    /// ```rust
    /// use hyperborealib::crypto::compression::{Compression, CompressionLevel};
    /// 
    /// let original = b"Example string with maaaaaaaaaaaaaaany repetitions";
    /// 
    /// let deflate = Compression::Deflate.compress(original, CompressionLevel::Quality).unwrap();
    /// let brotli  = Compression::Brotli.compress(original, CompressionLevel::Quality).unwrap();
    /// 
    /// assert!(deflate.len() > brotli.len());
    /// ```
    pub fn compress(&self, data: impl AsRef<[u8]>, level: CompressionLevel) -> Result<Vec<u8>, Error> {
        let data = data.as_ref();

        match self {
            Self::None => Ok(data.to_vec()),

            Self::Deflate => deflate::compress(data, level)
                .map_err(|err| Error::Compression(err.into())),

            Self::Brotli => brotli::compress(data, level)
                .map_err(|err| Error::Compression(err.into()))
        }
    }

    /// Decompress given data with selected compression algorithm.
    /// 
    /// ```rust
    /// use hyperborealib::crypto::compression::{Compression, CompressionLevel};
    /// 
    /// let original = b"Example string with maaaaaaaaaaaaaaany repetitions";
    /// 
    /// let compression = Compression::Deflate;
    /// 
    /// let compressed   = compression.compress(original, CompressionLevel::default()).unwrap();
    /// let decompressed = compression.decompress(&compressed).unwrap();
    /// 
    /// assert!(original.len() > compressed.len());
    /// assert_eq!(decompressed, original);
    /// ```
    pub fn decompress(&self, data: impl AsRef<[u8]>) -> Result<Vec<u8>, Error> {
        let data = data.as_ref();

        match self {
            Self::None => Ok(data.to_vec()),

            Self::Deflate => deflate::decompress(data)
                .map_err(|err| Error::Decompression(err.into())),

            Self::Brotli => brotli::decompress(data)
                .map_err(|err| Error::Decompression(err.into()))
        }
    }
}

impl std::str::FromStr for Compression {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "none" | "plain" => Ok(Self::None),

            "deflate" => Ok(Self::Deflate),
            "brotli"  => Ok(Self::Brotli),

            _ => Err(Error::UnknownCompression(value.to_string()))
        }
    }
}

impl std::fmt::Display for Compression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None    => write!(f, "plain"),
            Self::Deflate => write!(f, "deflate"),
            Self::Brotli  => write!(f, "brotli")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn compressions() -> &'static [(Compression, &'static str)] {
        &[
            (Compression::None,    "plain"),
            (Compression::Deflate, "deflate"),
            (Compression::Brotli,  "brotli")
        ]
    }

    fn levels() -> &'static [CompressionLevel] {
        &[
            CompressionLevel::Fast,
            CompressionLevel::Balanced,
            CompressionLevel::Quality
        ]
    }

    #[test]
    fn compress_decompress() -> Result<(), Error> {
        for (compression, _) in compressions() {
            for level in levels() {
                let compressed = compression.compress(b"Hello, World!", *level)?;
                let decompressed = compression.decompress(compressed)?;

                assert_eq!(decompressed, b"Hello, World!");
            }
        }

        Ok(())
    }

    #[test]
    fn display() {
        for (compression, name) in compressions() {
            assert_eq!(compression.to_string(), *name);
        }
    }

    #[test]
    fn parse() -> Result<(), Error> {
        for (compression, name) in compressions() {
            assert_eq!(name.parse::<Compression>()?, *compression);
        }

        Ok(())
    }
}
