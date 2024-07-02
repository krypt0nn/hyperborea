use serde_json::Value as Json;

use hyperborealib::http::HttpClient;
use hyperborealib::rest_api::prelude::*;

use crate::params::Params;

pub async fn process<T: HttpClient, F: std::future::Future<Output = anyhow::Result<Option<Json>>>>(
    client: &ClientMiddleware<T>,
    params: &Params,
    chat_sender: &Sender,
    channel: &str,
    callback: impl (FnOnce(Json, MessageInfo) -> F) + Clone
) -> anyhow::Result<()> {
    // Poll incoming messages
    let (messages, _) = client.poll(&params.server_local_address, channel, None).await
        .map_err(|err| anyhow::anyhow!(err.to_string()))?;

    // Parse message encoding from params
    let encoding = MessageEncoding::from_str(&params.room_encoding)
        .map_err(|err| anyhow::anyhow!(err.to_string()))?;

    for message in messages {
        // Read message
        let content = match message.message.read(
            client.driver().secret_key(),
            &message.sender.client.public_key
        ) {
            Ok(content) => content,

            Err(err) => {
                log::warn!(
                    "[process] failed to verify message: {err}. Client: {}, server: {} ({})",
                    message.sender.client.public_key.to_base64(),
                    message.sender.server.public_key.to_base64(),
                    message.sender.server.address
                );

                continue;
            }
        };

        // Deserialize message to get a request
        let request = match serde_json::from_slice::<Json>(&content) {
            Ok(request) => request,

            Err(err) => {
                log::warn!(
                    "[process] failed to deserialize message: {err}. Client: {}, server: {} ({})",
                    message.sender.client.public_key.to_base64(),
                    message.sender.server.public_key.to_base64(),
                    message.sender.server.address
                );

                continue;
            }
        };

        // Process the request and send back a response if needed
        if let Some(response) = (callback.clone())(request, message.clone()).await? {
            let response = Message::create(
                client.driver().secret_key(),
                &message.sender.client.public_key,
                serde_json::to_vec(&response)?,
                encoding
            ).map_err(|err| anyhow::anyhow!(err.to_string()))?;

            client.send(
                &message.sender.server.address,
                message.sender.client.public_key.clone(),
                chat_sender.clone(),
                channel,
                response
            ).await.map_err(|err| anyhow::anyhow!(err.to_string()))?;
        }
    }

    Ok(())
}
