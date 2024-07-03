pub mod client;

pub mod prelude {
    pub use hyperborealib;

    pub use super::client::{
        ClientParams,
        ClientEndpoint,
        ClientApp,
        ClientAppError
    };

    pub use super::build_client;
}

pub mod exports {
    pub use hyperborealib;
    pub use tokio;
}
