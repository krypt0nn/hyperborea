mod params;

#[allow(clippy::module_inception)]
mod server;

pub mod router;
pub mod traversal;
pub mod messages_inbox;

pub use params::ServerParams;
pub use server::ServerDriver;

pub mod prelude {
    pub use super::{
        ServerDriver,
        ServerParams
    };

    pub use super::router::Router;
    pub use super::traversal::Traversal;
    pub use super::messages_inbox::MessagesInbox;

    #[cfg(feature = "router-global-table")]
    pub use super::router::global_table::GlobalTableRouter;

    #[cfg(feature = "traversal-bfs-recursion")]
    pub use super::traversal::bfs_recursion::BfsRecursionTraversal;

    #[cfg(feature = "inbox-stored-queue")]
    pub use super::messages_inbox::stored_queue::StoredQueueMessagesInbox;
}
