use hyperelm::prelude::*;
use hyperborealib::prelude::*;

use crate::params::Params;

pub struct ServerApp {
    params: Params
}

impl ServerApp {
    #[inline]
    pub fn from_params(params: Params) -> Self {
        Self {
            params
        }
    }
}

impl BasicServerApp for ServerApp {
    fn get_params(&self) -> ServerAppParams {
        let secret_key = SecretKey::from_base64(&self.params.server_secret)
            .expect("Failed to deserialize server secret key");

        ServerAppParams {
            secret_key,
            local_address: self.params.server_local_address.clone(),
            remote_address: self.params.server_exposed_address.clone(),
            bootstrap: self.params.bootstrap_addresses.clone(),
            announce: false,
            traverse_delay: std::time::Duration::from_secs(self.params.bootstrap_traversal_delay)
        }
    }
}
