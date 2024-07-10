use std::time::Duration;

use hyperborealib::crypto::*;
use hyperborealib::rest_api::prelude::*;

#[derive(Debug, Clone)]
pub struct ClientAppParams {
    /// Secret key of the current client.
    pub client_secret: SecretKey,

    /// Public key of the server to connect to.
    pub server_public: PublicKey,

    /// Address of the server to connect to.
    pub server_address: String,

    /// Messaging channel.
    pub channel: String,

    /// Messages encoding format.
    pub encoding: MessageEncoding,

    /// Messages synchronization delay.
    pub delay: Duration
}

impl ClientAppParams {
    #[inline]
    pub fn builder() -> ClientAppParamsBuilder {
        ClientAppParamsBuilder::default()
    }
}

#[derive(Debug, Clone)]
pub struct ClientAppParamsBuilder {
    /// Secret key of the current client.
    pub client_secret: Option<SecretKey>,

    /// Public key of the server to connect to.
    pub server_public: Option<PublicKey>,

    /// Address of the server to connect to.
    pub server_address: Option<String>,

    /// Messaging channel.
    pub channel: String,

    /// Messages encoding format.
    pub encoding: MessageEncoding,

    /// Messages synchronization delay.
    pub delay: Duration
}

impl Default for ClientAppParamsBuilder {
    fn default() -> Self {
        Self {
            client_secret: None,
            server_public: None,
            server_address: None,
            channel: String::from("hyperelm"),
            encoding: MessageEncoding::default(),
            delay: Duration::from_secs(1)
        }
    }
}

impl ClientAppParamsBuilder {
    pub fn client(mut self, secret_key: SecretKey) -> Self {
        self.client_secret = Some(secret_key);

        self
    }

    pub fn server(mut self, public_key: PublicKey, address: impl ToString) -> Self {
        self.server_public = Some(public_key);
        self.server_address = Some(address.to_string());

        self
    }

    pub fn channel(mut self, channel: impl ToString) -> Self {
        self.channel = channel.to_string();

        self
    }

    pub fn encoding(mut self, encoding: MessageEncoding) -> Self {
        self.encoding = encoding;

        self
    }

    pub fn delay(mut self, delay: Duration) -> Self {
        self.delay = delay;

        self
    }

    pub fn build(self) -> Option<ClientAppParams> {
        Some(ClientAppParams {
            client_secret: self.client_secret?,
            server_public: self.server_public?,
            server_address: self.server_address?,
            channel: self.channel,
            encoding: self.encoding,
            delay: self.delay
        })
    }
}
