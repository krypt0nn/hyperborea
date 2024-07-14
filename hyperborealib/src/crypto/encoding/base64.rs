use base64::Engine;
use base64::engine::GeneralPurpose as Base64Engine;

lazy_static::lazy_static! {
    pub static ref BASE64: Base64Engine = Base64Engine::new(
        &base64::alphabet::STANDARD,
        base64::engine::GeneralPurposeConfig::default()
    );
}

#[inline]
/// Encode given binary data to the 64 base number.
/// 
/// ```rust
/// use hyperborealib::crypto::encoding::base64;
/// 
/// assert_eq!(base64::encode(b"Hello, World!"), "SGVsbG8sIFdvcmxkIQ==");
/// ```
pub fn encode(bytes: impl AsRef<[u8]>) -> String {
    BASE64.encode(bytes)
}

#[inline]
/// Decode given base 64 number into a binary data.
/// 
/// ```rust
/// use hyperborealib::crypto::encoding::base64;
/// 
/// assert_eq!(base64::decode("SGVsbG8sIFdvcmxkIQ=="), Ok(b"Hello, World!".to_vec()));
/// ```
pub fn decode(string: impl AsRef<str>) -> Result<Vec<u8>, base64::DecodeError> {
    BASE64.decode(string.as_ref())
}

#[cfg(test)]
pub mod tests {
    use super::*;

    pub fn encode_decode() -> Result<(), base64::DecodeError> {
        assert_eq!(decode(encode(b"Hello, World!"))?, b"Hello, World!");

        Ok(())
    }
}
