use std::io::Write;

use flate2::Compression;
use flate2::write::{DeflateEncoder, DeflateDecoder};

pub fn compress(data: impl AsRef<[u8]>) -> std::io::Result<Vec<u8>> {
    let data = data.as_ref();

    let mut encoder = DeflateEncoder::new(
        Vec::with_capacity(data.len()),
        Compression::default()
    );

    encoder.write_all(data)?;

    encoder.finish()
}

pub fn decompress(data: impl AsRef<[u8]>) -> std::io::Result<Vec<u8>> {
    let data = data.as_ref();

    let mut decoder = DeflateDecoder::new(Vec::with_capacity(data.len()));

    decoder.write_all(data)?;

    decoder.finish()
}

#[cfg(test)]
pub mod tests {
    use super::*;

    pub fn compress_decompress() -> std::io::Result<()> {
        assert_eq!(decompress(compress(b"Hello, World!")?)?, b"Hello, World!");

        Ok(())
    }
}
