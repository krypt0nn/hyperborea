use brotli::{
    BrotliCompress,
    BrotliDecompress
};

use brotli::enc::BrotliEncoderParams;

use super::CompressionLevel;

impl From<CompressionLevel> for BrotliEncoderParams {
    fn from(value: CompressionLevel) -> Self {
        let mut params = BrotliEncoderParams::default();

        match value {
            CompressionLevel::Fast => {
                params.quality = 0;

                params.favor_cpu_efficiency = true;
            }

            CompressionLevel::Balanced => {
                params.quality = 6;
            }

            CompressionLevel::Quality => {
                params.quality = 11;

                params.large_window = true;
            }
        }

        params
    }
}

/// Compress given data using brotli compression algorithm.
/// 
/// # Example
/// 
/// ```rust
/// use hyperborealib::crypto::compression::{CompressionLevel, brotli};
/// 
/// let original = b"Example string with maaaaaaaaaaaaaaany repetitions";
/// 
/// let compressed = brotli::compress(original, CompressionLevel::default()).unwrap();
/// 
/// assert!(original.len() > compressed.len());
/// ```
pub fn compress(data: impl AsRef<[u8]>, level: CompressionLevel) -> std::io::Result<Vec<u8>> {
    let mut data = data.as_ref();
    let mut compressed = Vec::with_capacity(data.len());

    let mut params = BrotliEncoderParams::from(level);

    params.size_hint = data.len();

    BrotliCompress(&mut data, &mut compressed, &params)?;

    Ok(compressed)
}

/// Decompress given data using brotli compression algorithm.
/// 
/// # Example
/// 
/// ```rust
/// use hyperborealib::crypto::compression::{CompressionLevel, brotli};
/// 
/// let original = b"Example string with maaaaaaaaaaaaaaany repetitions";
/// 
/// let compressed = brotli::compress(original, CompressionLevel::default()).unwrap();
/// let decompressed = brotli::decompress(compressed).unwrap();
/// 
/// assert_eq!(decompressed, original);
/// ```
pub fn decompress(data: impl AsRef<[u8]>) -> std::io::Result<Vec<u8>> {
    let mut data = data.as_ref();
    let mut decompressed = Vec::with_capacity(data.len());

    BrotliDecompress(&mut data, &mut decompressed)?;

    Ok(decompressed)
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
