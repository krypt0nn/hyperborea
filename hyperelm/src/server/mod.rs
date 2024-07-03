use hyperborealib::drivers::prelude::*;
use hyperborealib::rest_api::prelude::*;

mod params;
mod app;
mod basic_app;

pub use params::*;
pub use app::*;
pub use basic_app::*;

/// Start given server application in tokio async thread,
/// returning back an `Arc` containing original variant
/// of the server.
pub async fn run<T>(app: T) -> Result<(), T::Error>
where T: ServerApp + Send + Sync + 'static {
    let params = app.get_params();

    // Create client middlewire for traversal thread
    let traversal_client = ClientMiddleware::new(
        app.get_http_client()?,
        ClientDriver::new(ClientInfo::thin(), params.secret_key)
    );

    // Resolve server middlewire and driver
    let middlewire = app.get_middlewire().await?;
    let driver = middlewire.driver();

    // Start the server
    tokio::spawn(async move {
        if let Err(_err) = middlewire.serve(&params.local_address).await {
            #[cfg(feature = "tracing")]
            tracing::error!("[server] {_err}");
        }
    });

    loop {
        // Index bootstrap servers
        #[cfg(feature = "tracing")]
        tracing::debug!("[server] Indexing bootstrap addresses");

        for address in &params.bootstrap {
            if let Ok(server) = traversal_client.get_info(&address).await {
                driver.router().index_server(Server::new(
                    server.public_key,
                    address
                )).await;
            }
        }

        // Traverse network
        #[cfg(feature = "tracing")]
        tracing::debug!("[server] Traversing network");

        driver.traversal().traverse(
            traversal_client.http_client().clone(),
            &driver
        ).await;

        // Announce servers about ourselves
        if params.announce {
            // TODO
        }

        // Wait before repeating
        tokio::time::sleep(params.traverse_delay).await;
    }
}
