use crate::http::client::HttpClient;
use crate::server::router::Router;

use super::{
    Traversal,
    Server
};

pub struct BfsRecursionTraversal;

#[async_trait::async_trait]
impl Traversal for BfsRecursionTraversal {
    async fn traverse<T: HttpClient>(&self, server: &Server<T>) {
        // let client = server.as_client();

        // let mut servers = server.router().servers().await;

        // while let Some(server) = servers.pop() {
        //     // client.
        // }
    }
}
