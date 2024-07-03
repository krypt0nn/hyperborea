use std::time::Duration;

use hyperborealib::crypto::SecretKey;
use hyperborealib::drivers::prelude::*;
use hyperborealib::rest_api::prelude::*;

#[derive(Debug, Clone)]
pub struct ClientParams {
    /// Current client.
    pub client: ClientDriver,

    /// Current client's server.
    pub server: ServerDriver,

    /// Messaging channel.
    pub channel: String,

    /// Messages encoding format.
    pub encoding: MessageEncoding,

    /// Messages synchronization delay.
    pub delay: Duration
}

impl ClientParams {
    #[inline]
    pub fn builder() -> ClientParamsBuilder {
        ClientParamsBuilder::default()
    }
}

#[derive(Debug, Clone)]
pub struct ClientParamsBuilder {
    /// Current client.
    pub client: Option<ClientDriver>,

    /// Current client's server.
    pub server: Option<ServerDriver>,

    /// Messaging channel.
    pub channel: String,

    /// Messages encoding format.
    pub encoding: MessageEncoding,

    /// Messages synchronization delay.
    pub delay: Duration
}

impl Default for ClientParamsBuilder {
    fn default() -> Self {
        Self {
            client: None,
            server: None,
            channel: String::from("hyperelm"),
            encoding: MessageEncoding::new(
                TextEncoding::Base64,
                TextEncryption::ChaCha20Poly1305,
                TextCompression::None
            ),
            delay: Duration::from_secs(1)
        }
    }
}

impl ClientParamsBuilder {
    pub fn client(mut self, secret_key: SecretKey) -> Self {
        self.client = Some(ClientDriver::thin(secret_key));

        self
    }

    pub fn server(mut self, secret_key: SecretKey, address: impl ToString) -> Self {
        self.server = Some(ServerDriver::new(
            GlobalTableRouter::default(),
            BfsRecursionTraversal,
            BasicInbox::default(),
            ServerParams {
                server_secret: secret_key,
                server_address: address.to_string()
            }
        ));

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

    pub fn build(self) -> Option<ClientParams> {
        Some(ClientParams {
            client: self.client?,
            server: self.server?,
            channel: self.channel,
            encoding: self.encoding,
            delay: self.delay
        })
    }
}
