use std::io::Write;

use flate2::Compression;
use flate2::write::{DeflateEncoder, DeflateDecoder};

use super::CompressionLevel;

impl From<CompressionLevel> for Compression {
    fn from(value: CompressionLevel) -> Self {
        match value {
            CompressionLevel::Fast     => Self::fast(),
            CompressionLevel::Balanced => Self::default(),
            CompressionLevel::Quality  => Self::best()
        }
    }
}

/// Compress given data using deflate compression algorithm.
/// 
/// # Example
/// 
/// ```rust
/// use hyperborealib::crypto::compression::{CompressionLevel, deflate};
/// 
/// let original = b"Example string with maaaaaaaaaaaaaaany repetitions";
/// 
/// let compressed = deflate::compress(original, CompressionLevel::default()).unwrap();
/// 
/// assert!(original.len() > compressed.len());
/// ```
pub fn compress(data: impl AsRef<[u8]>, level: CompressionLevel) -> std::io::Result<Vec<u8>> {
    let data = data.as_ref();

    let mut encoder = DeflateEncoder::new(
        Vec::with_capacity(data.len()),
        level.into()
    );

    encoder.write_all(data)?;

    encoder.finish()
}

/// Decompress given data using deflate compression algorithm.
/// 
/// # Example
/// 
/// ```rust
/// use hyperborealib::crypto::compression::{CompressionLevel, deflate};
/// 
/// let original = b"Example string with maaaaaaaaaaaaaaany repetitions";
/// 
/// let compressed = deflate::compress(original, CompressionLevel::default()).unwrap();
/// let decompressed = deflate::decompress(compressed).unwrap();
/// 
/// assert_eq!(decompressed, original);
/// ```
pub fn decompress(data: impl AsRef<[u8]>) -> std::io::Result<Vec<u8>> {
    let data = data.as_ref();

    let mut decoder = DeflateDecoder::new(Vec::with_capacity(data.len()));

    decoder.write_all(data)?;

    decoder.finish()
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    pub fn compress_decompress() -> std::io::Result<()> {
        let level = CompressionLevel::default();

        assert_eq!(decompress(compress(b"Hello, World!", level)?)?, b"Hello, World!");

        Ok(())
    }
}
