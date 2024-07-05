use std::sync::Arc;

use hyperborealib::rest_api::middleware::Error;

mod params;
mod endpoint;
mod app;
mod macros;

pub use params::*;
pub use endpoint::*;
pub use app::*;

/// Start given client application in tokio async thread,
/// returning back an `Arc` containing original variant
/// of the client to perform `send` and `request` calls.
/// 
/// This method will perform `connect` request to the server
/// specified in the application's params and return error
/// if this request fails.
/// 
/// This method doesn't freeze the caller's thread.
pub async fn run<T>(app: T) -> Result<Arc<T>, Error>
where
    T: ClientApp + Send + Sync + 'static,
    T::Error: std::fmt::Display
{
    let params = app.get_params();

    // Try connecting to the application's server
    app.get_middlewire().connect_to(
        &params.server.params().address,
        params.server.params().secret_key.public_key()
    ).await?;

    // Start background updates task
    let client = Arc::new(app);

    {
        let client = client.clone();

        tokio::spawn(async move {
            let params = client.get_params();

            loop {
                if let Err(_err) = client.update().await {
                    #[cfg(feature = "tracing")]
                    tracing::error!("[client] Update error: {_err}");
                }

                tokio::time::sleep(params.delay).await;
            }
        });
    }

    Ok(client)
}
