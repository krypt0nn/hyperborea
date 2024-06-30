use std::net::ToSocketAddrs;
use std::sync::Arc;

use crate::http::client::HttpClient;
use crate::http::server::HttpServer;

use crate::server::Server as ServerDriver;

use crate::rest_api::prelude::*;

#[derive(Debug, Clone, Hash)]
/// Server HTTP middleware
/// 
/// This struct is used to process HTTP REST API requests
/// to the inner server driver.
pub struct Server<HttpClient, HttpServer, Router, Traversal, MessagesInbox> {
    driver: Arc<ServerDriver<Router, Traversal, MessagesInbox>>,
    http_client: HttpClient,
    http_server: HttpServer
}

impl<T, F, Router, Traversal, MessagesInbox> Server<T, F, Router, Traversal, MessagesInbox>
where
    T: HttpClient,
    F: HttpServer,
    Router: crate::server::router::Router + Send + Sync + 'static,
    Traversal: crate::server::traversal::Traversal + Send + Sync + 'static,
    MessagesInbox: crate::server::messages_inbox::MessagesInbox + Send + Sync + 'static,
{
    pub async fn new(http_client: T, mut http_server: F, server_driver: ServerDriver<Router, Traversal, MessagesInbox>) -> Self {
        #[cfg(feature = "tracing")]
        tracing::trace!(
            http_client_type = std::any::type_name::<T>(),
            http_server_type = std::any::type_name::<F>(),
            router_type = std::any::type_name::<Router>(),
            traversal_type = std::any::type_name::<Traversal>(),
            server_address = server_driver.params().server_address,
            server_secret = server_driver.params().server_secret.to_base64(),
            "Building server REST API middleware"
        );

        let driver = Arc::new(server_driver);

        http_server.get("/api/v1/info", {
            let driver = driver.clone();

            |client_address| async move {
                #[cfg(feature = "tracing")]
                tracing::trace!(?client_address, "GET /api/v1/info");

                InfoResponse::new(&driver.params().server_secret)
            }
        }).await;

        http_server.get("/api/v1/clients", {
            let driver = driver.clone();

            |client_address| async move {
                #[cfg(feature = "tracing")]
                tracing::trace!(?client_address, "GET /api/v1/clients");

                let clients = driver.router()
                    .local_clients().await;

                #[cfg(feature = "tracing")]
                tracing::trace!("GET /api/v1/clients: returned {} records", clients.len());

                ClientsResponse::new(clients)
            }
        }).await;

        http_server.get("/api/v1/servers", {
            let driver = driver.clone();

            |client_address| async move {
                #[cfg(feature = "tracing")]
                tracing::trace!(?client_address, "GET /api/v1/servers");

                let servers = driver.router()
                    .servers().await;

                #[cfg(feature = "tracing")]
                tracing::trace!("GET /api/v1/servers: returned {} records", servers.len());

                ServersResponse::new(servers)
            }
        }).await;

        http_server.post::<ConnectRequest, ConnectResponse, _>("/api/v1/connect", {
            let driver = driver.clone();

            |client_address, request| async move {
                #[cfg(feature = "tracing")]
                tracing::trace!(?client_address, "POST /api/v1/connect");

                // Validate incoming request
                let validated = match request.validate(&driver.params().server_secret.public_key()) {
                    Ok(validated) => validated,

                    Err(err) => return ConnectResponse::error(
                        ResponseStatus::ServerError,
                        format!("An error occured during request validation: {err}")
                    )
                };

                // Check if request is valid
                if !validated {
                    return ConnectResponse::error(
                        ResponseStatus::RequestValidationFailed,
                        "Request validation failed"
                    );
                }

                // Index client in the routing table
                let client = Client::new(
                    request.0.public_key,
                    request.0.request.certificate,
                    request.0.request.client
                );

                #[cfg(feature = "tracing")]
                tracing::trace!(
                    client_public = client.public_key.to_base64(),
                    client_info = std::any::type_name_of_val(&client.info),
                    "POST /api/v1/connect: indexing local client"
                );

                driver.router().index_local_client(client).await;

                ConnectResponse::success(
                    ResponseStatus::Success,
                    &driver.params().server_secret,
                    request.0.proof_seed
                )
            }
        }).await;

        http_server.post::<LookupRequest, LookupResponse, _>("/api/v1/lookup", {
            let driver = driver.clone();

            |client_address, request| async move {
                #[cfg(feature = "tracing")]
                tracing::trace!(?client_address, "POST /api/v1/lookup");

                // Validate incoming request
                let validated = match request.validate() {
                    Ok(validated) => validated,

                    Err(err) => return LookupResponse::error(
                        ResponseStatus::ServerError,
                        format!("An error occured during request validation: {err}")
                    )
                };

                // Check if request is valid
                if !validated {
                    return LookupResponse::error(
                        ResponseStatus::RequestValidationFailed,
                        "Request validation failed"
                    );
                }

                // Try to find the client in the local index
                if let Some(client) = driver.router().lookup_local_client(&request.0.public_key, request.0.request.client_type).await {
                    let body = LookupResponseBody::local(client, true);

                    return LookupResponse::success(
                        ResponseStatus::Success,
                        &driver.params().server_secret,
                        request.0.proof_seed,
                        body
                    );
                }

                // Try to find the client in the remote index
                if let Some((client, server)) = driver.router().lookup_remote_client(&request.0.public_key, request.0.request.client_type).await {
                    let body = LookupResponseBody::remote(client, server, true);

                    return LookupResponse::success(
                        ResponseStatus::Success,
                        &driver.params().server_secret,
                        request.0.proof_seed,
                        body
                    );
                }

                // Return searching hint if neither local nor known remote record found
                let hint = driver.router()
                    .lookup_remote_client_hint(&request.0.public_key, request.0.request.client_type)
                    .await;

                LookupResponse::success(
                    ResponseStatus::Success,
                    &driver.params().server_secret,
                    request.0.proof_seed,
                    LookupResponseBody::hint(hint)
                )
            }
        }).await;

        http_server.post::<SendRequest, SendResponse, _>("/api/v1/send", {
            let driver = driver.clone();

            |client_address, request| async move {
                #[cfg(feature = "tracing")]
                tracing::trace!(?client_address, "POST /api/v1/send");

                // Validate incoming request
                let validated = match request.validate() {
                    Ok(validated) => validated,

                    Err(err) => return SendResponse::error(
                        ResponseStatus::ServerError,
                        format!("An error occured during request validation: {err}")
                    )
                };

                // Check if request is valid
                if !validated {
                    return SendResponse::error(
                        ResponseStatus::RequestValidationFailed,
                        "Request validation failed"
                    );
                }

                // Add message to the inbox
                driver.messages_inbox().add_message(
                    request.0.request.sender,
                    request.0.request.receiver_public,
                    request.0.request.channel,
                    request.0.request.message
                ).await;

                SendResponse::success(
                    ResponseStatus::Success,
                    &driver.params().server_secret,
                    request.0.proof_seed
                )
            }
        }).await;

        Self {
            http_client,
            http_server,
            driver
        }
    }

    #[inline]
    /// Run HTTP REST API server on given TCP listener
    pub async fn serve(self, address: impl ToSocketAddrs + Send) -> Result<(), Box<dyn std::error::Error>> {
        #[cfg(feature = "tracing")]
        tracing::debug!("Starting server");

        self.http_server.serve(address).await
    }
}
