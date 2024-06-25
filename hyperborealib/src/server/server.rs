use crate::client::Client;

use crate::rest_api::connect::ClientInfo;

use super::router::global_table::GlobalTableRouter;
use super::traversal::bfs_recursion::BfsRecursionTraversal;

use super::params::ServerParams;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Server<Router = GlobalTableRouter, Traversal = BfsRecursionTraversal> {
    router: Router,
    traversal: Traversal,
    params: ServerParams
}

impl<Router, Traversal> Server<Router, Traversal>
where
    Router: crate::server::router::Router,
    Traversal: crate::server::traversal::Traversal
{
    #[inline]
    pub fn new(router: Router, traversal: Traversal, params: ServerParams) -> Self {
        Self {
            router,
            traversal,
            params
        }
    }

    #[inline]
    pub fn router(&self) -> &Router {
        &self.router
    }

    #[inline]
    pub fn traversal(&self) -> &Traversal {
        &self.traversal
    }

    #[inline]
    pub fn params(&self) -> &ServerParams {
        &self.params
    }

    /// Make `ClientInfo::Server` client from the current server
    pub fn as_client(&self) -> Client {
        Client::new(
            ClientInfo::server(&self.params.server_address),
            self.params.server_secret.clone()
        )
    }
}
