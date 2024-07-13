pub mod time;
pub mod crypto;
pub mod http;
pub mod drivers;
pub mod rest_api;

pub mod exports {
    pub use k256;
    pub use rand_chacha;
    pub use base64;

    #[cfg(feature = "client-reqwest")]
    pub use reqwest;

    #[cfg(feature = "server-axum")]
    pub use axum;

    #[cfg(feature = "server-axum")]
    pub use tokio;
}

pub const STANDARD_VERSION: u64 = 1;
pub const LIBRARY_VERSION: &str = env!("CARGO_PKG_VERSION");
