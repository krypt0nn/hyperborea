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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compress_decompress() -> Result<(), Error> {
        let compressions = [
            Compression::None,
            Compression::Deflate,
            Compression::Brotli
        ];

        let levels = [
            CompressionLevel::Fast,
            CompressionLevel::Balanced,
            CompressionLevel::Quality
        ];

        for compression in compressions {
            for level in levels {
                let compressed = compression.compress(b"Hello, World!", level)?;
                let decompressed = compression.decompress(compressed)?;

                assert_eq!(decompressed, b"Hello, World!");
            }
        }

        Ok(())
    }
}
