use super::Error;

pub mod deflate;

pub mod prelude {
    pub use super::Compression;

    pub use super::deflate::{
        compress as deflate_compress,
        decompress as deflate_decompress
    };
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Compression {
    #[default]
    None,

    Deflate
}

impl Compression {
    pub fn compress(&self, data: impl AsRef<[u8]>) -> Result<Vec<u8>, Error> {
        let data = data.as_ref();

        match self {
            Self::None => Ok(data.to_vec()),

            Self::Deflate => deflate::compress(data)
                .map_err(|err| Error::Compression(err.into()))
        }
    }

    pub fn decompress(&self, data: impl AsRef<[u8]>) -> Result<Vec<u8>, Error> {
        let data = data.as_ref();

        match self {
            Self::None => Ok(data.to_vec()),

            Self::Deflate => deflate::decompress(data)
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
            Compression::Deflate
        ];

        for compression in compressions {
            let compressed = compression.compress(b"Hello, World!")?;
            let decompressed = compression.decompress(compressed)?;

            assert_eq!(decompressed, b"Hello, World!");
        }

        Ok(())
    }
}
