use hyperborealib::http::*;
use hyperborealib::rest_api::prelude::*;
use hyperborealib::drivers::prelude::*;

use super::ServerAppParams;

#[async_trait::async_trait]
pub trait ServerApp {
    type Router: Router + Send + Sync + 'static;
    type Traversal: Traversal + Send + Sync + 'static;
    type MessagesInbox: MessagesInbox + Send + Sync + 'static;

    type HttpClient: HttpClient + Send + Sync + 'static;
    type HttpServer: HttpServer + Send + Sync + 'static;

    type Error: std::error::Error + Send;

    fn get_router(&self) -> Result<Self::Router, Self::Error>;
    fn get_traversal(&self) -> Result<Self::Traversal, Self::Error>;
    fn get_messages_inbox(&self) -> Result<Self::MessagesInbox, Self::Error>;

    fn get_http_client(&self) -> Result<Self::HttpClient, Self::Error>;
    fn get_http_server(&self) -> Result<Self::HttpServer, Self::Error>;

    fn get_params(&self) -> ServerAppParams;

    #[allow(clippy::type_complexity)]
    fn get_driver(&self) -> Result<ServerDriver<
        Self::Router,
        Self::Traversal,
        Self::MessagesInbox
    >, Self::Error> {
        let params = self.get_params();

        Ok(ServerDriver::new(
            self.get_router()?,
            self.get_traversal()?,
            self.get_messages_inbox()?,
            ServerParams {
                secret_key: params.secret_key.clone(),
                address: params.remote_address.clone()
            }
        ))
    }

    #[allow(clippy::type_complexity)]
    async fn get_middlewire(&self) -> Result<ServerMiddleware<
        Self::HttpClient,
        Self::HttpServer,
        Self::Router,
        Self::Traversal,
        Self::MessagesInbox
    >, Self::Error> {
        Ok(ServerMiddleware::new(
            self.get_http_client()?,
            self.get_http_server()?,
            self.get_driver()?
        ).await)
    }
}
