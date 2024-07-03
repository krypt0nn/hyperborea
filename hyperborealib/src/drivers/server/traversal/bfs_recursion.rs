use std::collections::VecDeque;

use crate::http::client::HttpClient;
use crate::rest_api::middleware::Client as ClientMiddleware;

use super::*;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BfsRecursionTraversal;

#[async_trait::async_trait]
impl Traversal for BfsRecursionTraversal {
    async fn traverse<R, T, I>(&self, http_client: impl HttpClient, server: &ServerDriver<R, T, I>)
    where
        R: Router + Sync,
        T: Traversal + Sync,
        I: MessagesInbox + Sync
    {
        let client = ClientMiddleware::new(http_client, server.as_client());

        let mut remote_servers = VecDeque::from(server.router().servers().await);

        while let Some(remote_server) = remote_servers.pop_front() {
            if let Ok(mut response) = client.get_servers(&remote_server.address).await {
                for remote_server in response.drain(..) {
                    remote_servers.push_back(remote_server);
                }
            }

            server.router().index_server(remote_server).await;
        }
    }
}
