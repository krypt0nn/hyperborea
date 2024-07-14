use crate::crypto::asymmetric::PublicKey;

use crate::rest_api::clients::Client;
use crate::rest_api::servers::Server;
use crate::rest_api::connect::ClientType;

pub mod global_table;

#[async_trait::async_trait]
/// Router is a struct that implements network clients
/// and servers indexing, listing and lookup operations.
pub trait Router {
    /// Index local client in the routing table
    async fn index_local_client(&self, client: Client);

    /// Index remote client in the routing table
    async fn index_remote_client(&self, client: Client, _server: Server) {
        self.index_local_client(client).await
    }

    /// Index server in the routing table
    async fn index_server(&self, server: Server);

    /// Get list of all connected local clients
    async fn local_clients(&self) -> Vec<Client>;

    /// Get list of all known remote clients and their servers
    async fn remote_clients(&self) -> Vec<(Client, Server)>;

    /// Get list of all known servers
    async fn servers(&self) -> Vec<Server> {
        self.remote_clients().await
            .iter()
            .map(|(_, server)| server)
            .cloned()
            .collect::<Vec<_>>()
    }

    /// Lookup local client in the routing table
    async fn lookup_local_client(&self, public_key: &PublicKey, client_type: Option<ClientType>) -> Option<Client> {
        self.local_clients().await
            .iter()
            .filter(|client| &client.public_key == public_key)
            .find(|client| client_type.is_none() || client_type == Some(client.info.client_type))
            .cloned()
    }

    /// Lookup remote client in the routing table
    async fn lookup_remote_client(&self, public_key: &PublicKey, client_type: Option<ClientType>) -> Option<(Client, Server)> {
        self.remote_clients().await
            .iter()
            .filter(|(client, _)| &client.public_key == public_key)
            .find(|(client, _)| client_type.is_none() || client_type == Some(client.info.client_type))
            .cloned()
    }

    /// Get list of servers which can know the client with given public key
    async fn lookup_remote_client_hint(&self, _public_key: &PublicKey, _client_type: Option<ClientType>) -> Vec<Server> {
        self.servers().await
    }

    /// Lookup server in the routing table
    async fn lookup_server(&self, public_key: &PublicKey) -> Option<Server> {
        self.servers().await
            .iter()
            .find(|server| &server.public_key == public_key)
            .cloned()
    }
}
