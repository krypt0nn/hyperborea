mod secret_key;
mod public_key;

pub use secret_key::SecretKey;
pub use public_key::PublicKey;

pub const HKDF_INFO: [u8; 64] = [
    162, 241, 203, 77,  49,  90,  31,  126, 67,  94,  191, 219, 56,  141, 46,  233,
    70,  18,  207, 194, 52,  154, 176, 139, 244, 222, 155, 110, 177, 91,  32,  218,
    150, 232, 148, 23,  13,  172, 48,  131, 95,  216, 144, 224, 163, 106, 254, 135,
    93,  220, 84,  116, 42,  3,   211, 57,  186, 174, 208, 121, 253, 185, 210, 240
];

#[cfg(test)]
mod tests {
    use crate::crypto::prelude::*;

    #[test]
    fn serialize() -> Result<(), CryptographyError> {
        let secret = SecretKey::random();
        let public = secret.public_key();

        assert_eq!(SecretKey::deserialize(secret.serialize())?, secret);
        assert_eq!(PublicKey::from_bytes(public.to_bytes())?, public);

        Ok(())
    }

    #[test]
    fn base64() -> Result<(), CryptographyError> {
        let secret = SecretKey::random();
        let public = secret.public_key();

        assert_eq!(SecretKey::from_base64(secret.to_base64())?, secret);
        assert_eq!(PublicKey::from_base64(public.to_base64())?, public);

        Ok(())
    }

    #[test]
    fn shared_secret() {
        let secret_1 = SecretKey::random();
        let secret_2 = SecretKey::random();

        let key_1 = secret_1.create_shared_secret(&secret_2.public_key(), None);
        let key_2 = secret_2.create_shared_secret(&secret_1.public_key(), None);

        assert!(key_1 == key_2);
    }

    #[test]
    fn signature() -> Result<(), CryptographyError> {
        let secret = SecretKey::random();

        let message = b"Hello, World!";
        let signature = secret.create_signature(message);

        assert!(secret.public_key().verify_signature(message, signature)?);

        Ok(())
    }

    #[test]
    fn encapsulated_signature() -> Result<(), CryptographyError> {
        let secret = SecretKey::random();

        let signature = secret.create_encapsulated_signature(b"Hello, World!");

        assert_eq!(secret.public_key().decode_encapsulated_signature(signature)?, Some(b"Hello, World!".to_vec()));

        Ok(())
    }
}
