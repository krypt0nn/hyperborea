use serde_json::Value as Json;

use hyperborealib::http::HttpClient;
use hyperborealib::rest_api::prelude::*;

use crate::params::Params;

pub async fn listen<T: HttpClient, F: std::future::Future<Output = anyhow::Result<()>>>(
    client: &ClientMiddleware<T>,
    params: &Params,
    channel: &str,
    callback: impl (FnOnce(Json, MessageInfo) -> F) + Clone
) -> anyhow::Result<()> {
    // Poll incoming messages
    let (messages, _) = client.poll(&params.server_local_address, channel, None).await
        .map_err(|err| anyhow::anyhow!(err.to_string()))?;

    for message in messages {
        // Read message
        let content = match message.message.read(
            client.driver().secret_key(),
            &message.sender.client.public_key
        ) {
            Ok(content) => content,

            Err(err) => {
                log::warn!("[client/listen] Failed to verify message");
                log::warn!("[client/listen]   Client : {}", message.sender.client.public_key.to_base64());
                log::warn!("[client/listen]   Server : {} ({})", message.sender.server.public_key.to_base64(), message.sender.server.address);
                log::warn!("[client/listen]   Reason : {err}");

                continue;
            }
        };

        // Deserialize message to get an announcement
        let announcement = match serde_json::from_slice::<Json>(&content) {
            Ok(announcement) => announcement,

            Err(err) => {
                log::warn!("[client/listen] Failed to deserialize message");
                log::warn!("[client/listen]   Client : {}", message.sender.client.public_key.to_base64());
                log::warn!("[client/listen]   Server : {} ({})", message.sender.server.public_key.to_base64(), message.sender.server.address);
                log::warn!("[client/listen]   Reason : {err}");

                continue;
            }
        };

        // Process the announcement
        (callback.clone())(announcement, message.clone()).await?;
    }

    Ok(())
}
