use crate::crypto::SecretKey;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ServerParams {
    pub secret_key: SecretKey,

    /// Globally accessible address of this server.
    /// 
    /// This is needed when we perform requests
    /// from the server as a `server(addresss)` client.
    pub address: String
}

impl Default for ServerParams {
    fn default() -> Self {
        Self {
            secret_key: SecretKey::random(),
            address: String::from("127.0.0.1:8001")
        }
    }
}
