pub mod client;
pub mod server;

pub mod prelude {
    pub use hyperborealib;

    pub use super::client::{
        ClientAppParams,
        ClientEndpoint,
        ClientApp,
        ClientAppError
    };

    pub use super::server::{
        ServerApp,
        ServerAppParams
    };

    #[cfg(feature = "server-basic-app")]
    pub use super::server::BasicServerApp;

    pub use super::build_client;
}

pub mod exports {
    pub use hyperborealib;
    pub use tokio;
}
