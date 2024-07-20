use crate::crypto::asymmetric::PublicKey;
use crate::rest_api::prelude::*;

#[cfg(feature = "router-global-table")]
pub mod global_table;

#[async_trait::async_trait]
/// Router is a struct that implements network clients
/// and servers indexing, listing and lookup operations.
pub trait Router {
    type Error: std::error::Error + Send + Sync;

    /// Index local client in the routing table.
    /// 
    /// This method will return whether the client was indexed.
    async fn index_local_client(&self, client: Client) -> Result<bool, Self::Error>;

    /// Index remote client in the routing table.
    /// 
    /// This method will return whether the client was indexed.
    async fn index_remote_client(&self, client: Client, _server: Server) -> Result<bool, Self::Error> {
        self.index_local_client(client).await
    }

    /// Index server in the routing table.
    /// 
    /// This method will return whether the server was indexed.
    async fn index_server(&self, server: Server) -> Result<bool, Self::Error>;

    /// Get list of all connected local clients.
    async fn local_clients(&self) -> Result<Vec<Client>, Self::Error>;

    /// Get list of all known remote clients and their servers.
    async fn remote_clients(&self) -> Result<Vec<(Client, Server)>, Self::Error>;

    /// Get list of all known servers.
    async fn servers(&self) -> Result<Vec<Server>, Self::Error> {
        Ok(self.remote_clients().await?
            .iter()
            .map(|(_, server)| server)
            .cloned()
            .collect::<Vec<_>>())
    }

    /// Lookup local client in the routing table.
    /// 
    /// Router can return optional availability field.
    async fn lookup_local_client(&self, public_key: &PublicKey, client_type: Option<ClientType>) -> Result<Option<(Client, bool)>, Self::Error> {
        Ok(self.local_clients().await?
            .iter()
            .filter(|client| &client.public_key == public_key)
            .find(|client| client_type.is_none() || client_type == Some(client.info.client_type))
            .cloned()
            .map(|client| (client, true)))
    }

    /// Lookup remote client in the routing table.
    /// 
    /// Router can return optional availability field.
    async fn lookup_remote_client(&self, public_key: &PublicKey, client_type: Option<ClientType>) -> Result<Option<(Client, Server, bool)>, Self::Error> {
        Ok(self.remote_clients().await?
            .iter()
            .filter(|(client, _)| &client.public_key == public_key)
            .find(|(client, _)| client_type.is_none() || client_type == Some(client.info.client_type))
            .cloned()
            .map(|(client, server)| (client, server, true)))
    }

    /// Get list of servers which can know the client with given public key.
    async fn lookup_remote_client_hint(&self, _public_key: &PublicKey, _client_type: Option<ClientType>) -> Result<Vec<Server>, Self::Error> {
        self.servers().await
    }

    /// Lookup server in the routing table.
    /// 
    /// Router can return optional availability field.
    async fn lookup_server(&self, public_key: &PublicKey) -> Result<Option<(Server, bool)>, Self::Error> {
        Ok(self.servers().await?
            .iter()
            .find(|server| &server.public_key == public_key)
            .cloned()
            .map(|server| (server, true)))
    }
}
