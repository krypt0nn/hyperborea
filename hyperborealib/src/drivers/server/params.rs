use crate::crypto::SecretKey;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ServerParams {
    pub server_secret: SecretKey,
    pub server_address: String
}

impl Default for ServerParams {
    fn default() -> Self {
        Self {
            server_secret: SecretKey::random(),
            server_address: String::from("localhost:80")
        }
    }
}
