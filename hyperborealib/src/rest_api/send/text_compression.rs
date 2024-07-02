use std::io::Write;

use flate2::Compression;
use flate2::write::{DeflateEncoder, DeflateDecoder};

use super::Error;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TextCompression {
    #[default]
    None,

    Deflate
}

impl TextCompression {
    pub fn compress(&self, data: impl AsRef<[u8]>) -> Result<Vec<u8>, Error> {
        let data = data.as_ref();

        match self {
            Self::None => Ok(data.to_vec()),

            Self::Deflate => {
                let mut encoder = DeflateEncoder::new(
                    Vec::with_capacity(data.len()),
                    Compression::default()
                );

                encoder.write_all(data)
                    .map_err(|err| Error::TextCompressionFailed(err.into()))?;

                encoder.finish().map_err(|err| Error::TextCompressionFailed(err.into()))
            }
        }
    }

    pub fn decompress(&self, data: impl AsRef<[u8]>) -> Result<Vec<u8>, Error> {
        let data = data.as_ref();

        match self {
            Self::None => Ok(data.to_vec()),

            Self::Deflate => {
                let mut decoder = DeflateDecoder::new(Vec::with_capacity(data.len()));

                decoder.write_all(data)
                    .map_err(|err| Error::TextDecompressionFailed(err.into()))?;

                decoder.finish().map_err(|err| Error::TextDecompressionFailed(err.into()))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compress_decompress() -> Result<(), Error> {
        let compressions = [
            TextCompression::None,
            TextCompression::Deflate
        ];

        for compression in compressions {
            let compressed = compression.compress(b"Hello, World!")?;
            let decompressed = compression.decompress(compressed)?;

            assert_eq!(decompressed, b"Hello, World!");
        }

        Ok(())
    }
}
