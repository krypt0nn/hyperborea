use crate::http::client::HttpClient;

use super::router::Router;
use super::Server;

pub mod bfs_recursion;

#[async_trait::async_trait]
/// Traversal is a struct that implements network servers
/// searching. It is called manually by the dev and intended
/// to keep the updated state of the network servers.
pub trait Traversal {
    /// Update network map using given server.
    async fn traverse<T: Router + Sync>(&self, http_client: impl HttpClient, server: &Server<T>);
}
