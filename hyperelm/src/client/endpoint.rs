use hyperborealib::crypto::PublicKey;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ClientEndpoint {
    pub server_address: String,
    pub client_public: PublicKey
}

impl ClientEndpoint {
    #[inline]
    pub fn new(server_address: impl ToString, client_public: PublicKey) -> Self {
        Self {
            server_address: server_address.to_string(),
            client_public
        }
    }
}
