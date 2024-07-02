use std::sync::Arc;

use tokio::sync::Mutex;

use serde_json::{json, Value as Json};

use hyperborealib::crypto::SecretKey;
use hyperborealib::http::{HttpClient, ReqwestHttpClient};
use hyperborealib::drivers::ClientDriver;
use hyperborealib::rest_api::prelude::*;

use crate::params::Params;

pub async fn run(params: &Params) -> anyhow::Result<()> {
    let server_secret = SecretKey::from_base64(&params.server_secret)?;
    let chat_secret = SecretKey::from_base64(&params.room_secret_key)?;

    let http = ReqwestHttpClient::default();

    let driver = ClientDriver::thin(chat_secret);
    let client = ClientMiddleware::new(http, driver);

    // Connect to the local server
    client.connect(&params.server_local_address).await
        .map_err(|err| anyhow::anyhow!(err.to_string()))?;

    // Sign the connection certificate
    let certificate = ConnectionCertificate::new(
        client.driver().secret_key(),
        server_secret.public_key()
    );

    // Prepare chat endpoint info for incoming requests
    let chat_sender = Sender::new(
        Client::new(client.driver().secret_key().public_key(), certificate, ClientInfo::thin()),
        Server::new(server_secret.public_key(), params.server_exposed_address.clone())
    );

    let users = Arc::new(Mutex::new(Vec::<(Sender, String)>::new()));

    // Start the chat room server
    loop {
        if let Err(err) = process_joins(&client, params, &chat_sender, users.clone()).await {
            log::error!("[join] {err}");
        };

        // Timeout til next update
        tokio::time::sleep(std::time::Duration::from_millis(params.room_sync_delay)).await;
    }
}

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

pub async fn process_joins<T: HttpClient>(
    client: &ClientMiddleware<T>,
    params: &Params,
    chat_sender: &Sender,
    users: Arc<Mutex<Vec<(Sender, String)>>>
) -> anyhow::Result<()> {
    process(client, params, chat_sender, "hyperchat/join", |request, info| async move {
        // Parse request fields
        let Some(username) = request["username"].as_str() else {
            log::warn!(
                "[join] wrong request format: no username field given. Client: {}, server: {} ({})",
                info.sender.client.public_key.to_base64(),
                info.sender.server.public_key.to_base64(),
                info.sender.server.address
            );

            return Ok(None);
        };

        // Filter username
        let username = username.chars()
            .filter(|c| !c.is_control())
            .collect::<String>();

        // Announce other users about newcomer
        log::info!("[join] {username} joins room");

        let join_announcement = serde_json::to_string(&json!({
            "client": info.sender.client,
            "server": info.sender.server,
            "username": username
        }))?;

        // Parse message encoding from params
        let encoding = MessageEncoding::from_str(&params.room_encoding)
            .map_err(|err| anyhow::anyhow!(err.to_string()))?;

        let mut users = users.lock().await;

        // Share join announcement with other chat members
        let mut i = 0;

        while i < users.len() {
            let (user, _) = &users[i];

            let message = Message::create(
                client.driver().secret_key(),
                &user.client.public_key,
                &join_announcement,
                encoding
            ).map_err(|err| anyhow::anyhow!(err.to_string()))?;

            let result = client.send(
                &user.server.address,
                user.client.public_key.clone(),
                chat_sender.clone(),
                "hyperchat/join",
                message
            ).await;

            if let Err(err) = result {
                log::info!(
                    "[join] failed to send join announcement to the client: {err}. Removing from the users list. Client: {}, server: {} ({})",
                    info.sender.client.public_key.to_base64(),
                    info.sender.server.public_key.to_base64(),
                    info.sender.server.address
                );

                users.remove(i);
            }

            else {
                i += 1;
            }
        }

        // Store the info about the new user
        users.push((
            info.sender,
            username
        ));

        Ok(Some(json!({
            "users": users.to_vec()
        })))
    }).await
}
