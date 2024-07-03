use std::sync::Arc;

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
pub async fn run<T>(app: T) -> Arc<T>
where T: ClientApp + Send + Sync + 'static {
    let client = Arc::new(app);

    {
        let client = client.clone();

        tokio::spawn(async move {
            let params = client.get_params();

            loop {
                if let Err(_err) = client.update().await {
                    #[cfg(feature = "tracing")]
                    tracing::error!("[client] {_err}");
                }

                tokio::time::sleep(params.delay).await;
            }
        });
    }

    client
}
