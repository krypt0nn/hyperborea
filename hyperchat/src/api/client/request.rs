use serde_json::Value as Json;

use hyperborealib::crypto::*;
use hyperborealib::http::HttpClient;
use hyperborealib::rest_api::prelude::*;

use crate::params::Params;

pub async fn request<T: HttpClient, F: std::future::Future<Output = anyhow::Result<()>>>(
    client: &ClientMiddleware<T>,
    params: &Params,
    server_address: &str,
    client_public: &PublicKey,
    channel: &str,
    request: &Json,
    callback: impl FnOnce(Json, MessageInfo) -> F
) -> anyhow::Result<()> {
    let server_secret = SecretKey::from_base64(&params.server_secret)?;

    // Sign the connection certificate
    let certificate = ConnectionCertificate::new(
        client.driver().secret_key(),
        server_secret.public_key()
    );

    // Prepare client endpoint info for incoming response
    let client_sender = Sender::new(
        Client::new(client.driver().secret_key().public_key(), certificate, ClientInfo::thin()),
        Server::new(server_secret.public_key(), params.server_exposed_address.clone())
    );

    // Parse message encoding from params
    let encoding = MessageEncoding::from_str(&params.room_encoding)
        .map_err(|err| anyhow::anyhow!(err.to_string()))?;

    // Prepare request message
    let request = Message::create(
        client.driver().secret_key(),
        client_public,
        serde_json::to_vec(&request)?,
        encoding
    ).map_err(|err| anyhow::anyhow!(err.to_string()))?;

    // Send request to the server
    client.send(
        server_address,
        client_public.to_owned(),
        client_sender,
        channel,
        request
    ).await.map_err(|err| anyhow::anyhow!(err.to_string()))?;

    // Await response
    loop {
        // Poll incoming message
        let (messages, _) = client.poll(&params.server_local_address, channel, Some(1)).await
            .map_err(|err| anyhow::anyhow!(err.to_string()))?;

        // Accept incoming response
        if let Some(message) = messages.first() {
            // Read message
            let content = match message.message.read(
                client.driver().secret_key(),
                &message.sender.client.public_key
            ) {
                Ok(content) => content,

                Err(err) => {
                    log::warn!("[client/request] Failed to verify response");
                    log::warn!("[client/request]   Client : {}", message.sender.client.public_key.to_base64());
                    log::warn!("[client/request]   Server : {} ({})", message.sender.server.public_key.to_base64(), message.sender.server.address);
                    log::warn!("[client/request]   Reason : {err}");

                    continue;
                }
            };

            // Deserialize message to get a response
            let response = match serde_json::from_slice::<Json>(&content) {
                Ok(response) => response,

                Err(err) => {
                    log::warn!("[client/request] Failed to deserialize response");
                    log::warn!("[client/request]   Client : {}", message.sender.client.public_key.to_base64());
                    log::warn!("[client/request]   Server : {} ({})", message.sender.server.public_key.to_base64(), message.sender.server.address);
                    log::warn!("[client/request]   Reason : {err}");

                    continue;
                }
            };

            // Process the response
            callback(response, message.clone()).await?;

            break;
        }

        else {
            // Timeout til next update
            tokio::time::sleep(std::time::Duration::from_millis(params.room_sync_delay)).await;
        }
    }

    Ok(())
}
