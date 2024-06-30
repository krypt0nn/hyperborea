mod params;
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

    pub use super::router::{
        Router,
        global_table::GlobalTableRouter
    };

    pub use super::traversal::{
        Traversal,
        bfs_recursion::BfsRecursionTraversal
    };

    pub use super::messages_inbox::{
        MessagesInbox,
        basic_inbox::BasicInbox
    };
}
