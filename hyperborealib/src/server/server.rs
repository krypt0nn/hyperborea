use crate::client::Client;

use crate::rest_api::connect::ClientInfo;

use super::router::global_table::GlobalTableRouter;
use super::traversal::bfs_recursion::BfsRecursionTraversal;
use super::messages_inbox::basic_inbox::BasicInbox;

use super::params::ServerParams;

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Server<
    Router = GlobalTableRouter,
    Traversal = BfsRecursionTraversal,
    MessagesInbox = BasicInbox
> {
    router: Router,
    traversal: Traversal,
    messages_inbox: MessagesInbox,
    params: ServerParams
}

impl<Router, Traversal, MessagesInbox> Server<Router, Traversal, MessagesInbox>
where
    Router: super::router::Router,
    Traversal: super::traversal::Traversal,
    MessagesInbox: super::messages_inbox::MessagesInbox
{
    #[inline]
    pub fn new(router: Router, traversal: Traversal, messages_inbox: MessagesInbox, params: ServerParams) -> Self {
        Self {
            router,
            traversal,
            messages_inbox,
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
    pub fn messages_inbox(&self) -> &MessagesInbox {
        &self.messages_inbox
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
