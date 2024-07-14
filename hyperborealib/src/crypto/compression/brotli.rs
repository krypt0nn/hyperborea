use std::io::Write;

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

pub fn compress(data: impl AsRef<[u8]>, level: CompressionLevel) -> std::io::Result<Vec<u8>> {
    let mut data = data.as_ref();
    let mut compressed = Vec::with_capacity(data.len());

    let mut params = BrotliEncoderParams::from(level);

    params.size_hint = data.len();

    BrotliCompress(&mut data, &mut compressed, &params)?;

    Ok(compressed)
}

pub fn decompress(data: impl AsRef<[u8]>) -> std::io::Result<Vec<u8>> {
    let mut data = data.as_ref();
    let mut decompressed = Vec::with_capacity(data.len());

    BrotliDecompress(&mut data, &mut decompressed)?;

    Ok(decompressed)
}

#[cfg(test)]
pub mod tests {
    use super::*;

    pub fn compress_decompress() -> std::io::Result<()> {
        let level = CompressionLevel::default();

        assert_eq!(decompress(compress(b"Hello, World!", level)?)?, b"Hello, World!");

        Ok(())
    }
}
