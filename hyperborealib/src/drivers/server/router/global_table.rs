use std::time::Duration;

use moka::future::Cache;

use crate::crypto::asymmetric::PublicKey;

use crate::rest_api::clients::Client;
use crate::rest_api::servers::Server;

use super::Router;

#[derive(Debug, Clone)]
pub struct GlobalTableRouter {
    pub local_clients: Cache<PublicKey, Client>,
    pub remote_clients: Cache<PublicKey, (Client, Server)>,
    pub servers: Cache<PublicKey, Server>
}

impl Default for GlobalTableRouter {
    #[inline]
    fn default() -> Self {
        Self::new(1024, Duration::from_secs(60 * 30))
    }
}

impl GlobalTableRouter {
    pub fn new(max_entries: u64, ttl: Duration) -> Self {
        #[cfg(feature = "tracing")]
        tracing::trace!("Building new GlobalTableRouter with {max_entries} max entries and {} seconds lifetime", ttl.as_secs());

        Self {
            local_clients: Cache::builder()
                .max_capacity(max_entries)
                .time_to_idle(ttl)
                .build(),

            remote_clients: Cache::builder()
                .max_capacity(max_entries)
                .time_to_idle(ttl)
                .build(),

            servers: Cache::builder()
                .max_capacity(max_entries)
                .time_to_idle(ttl)
                .build()
        }
    }
}

#[async_trait::async_trait]
impl Router for GlobalTableRouter {
    async fn index_local_client(&self, client: Client) {
        #[cfg(feature = "tracing")]
        tracing::debug!(
            client_public = client.public_key.to_base64(),
            client_info = ?client.info,
            "Indexing local client"
        );

        self.local_clients.insert(client.public_key.clone(), client).await;
    }

    async fn index_remote_client(&self, client: Client, server: Server) {
        #[cfg(feature = "tracing")]
        tracing::debug!(
            client_public = client.public_key.to_base64(),
            client_info = ?client.info,
            server_public = server.public_key.to_base64(),
            server_address = server.address,
            "Indexing remote client"
        );

        self.remote_clients.insert(client.public_key.clone(), (client, server)).await;
    }

    async fn index_server(&self, server: Server) {
        #[cfg(feature = "tracing")]
        tracing::debug!(
            server_public = server.public_key.to_base64(),
            server_address = server.address,
            "Indexing server"
        );

        self.servers.insert(server.public_key.clone(), server).await;
    }

    async fn local_clients(&self) -> Vec<Client> {
        self.local_clients.into_iter()
            .map(|(_, value)| value)
            .collect()
    }

    async fn remote_clients(&self) -> Vec<(Client, Server)> {
        self.remote_clients.into_iter()
            .map(|(_, value)| value)
            .collect()
    }

    async fn servers(&self) -> Vec<Server> {
        self.servers.into_iter()
            .map(|(_, value)| value)
            .collect()
    }
}
