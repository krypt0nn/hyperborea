use crate::http::client::HttpClient;

use super::prelude::*;

#[cfg(feature = "traversal-bfs-recursion")]
pub mod bfs_recursion;

#[async_trait::async_trait]
/// Traversal is a struct that implements network servers
/// searching. It is called manually by the dev and intended
/// to keep the updated state of the network servers.
pub trait Traversal {
    /// Update network map using given server.
    async fn traverse<R, T, I>(&self, http_client: impl HttpClient, server: &ServerDriver<R, T, I>)
    where
        R: Router + Sync,
        T: Traversal + Sync,
        I: MessagesInbox + Sync;
}
