pub mod bfs_recursion;

use crate::http::client::HttpClient;

use super::Server;

#[async_trait::async_trait]
/// Traversal is a struct that implements network servers
/// searching. It is called manually by the dev and intended
/// to keep the updated state of the network servers.
pub trait Traversal {
    async fn traverse<T: HttpClient>(&self, server: &Server<T>);
}
