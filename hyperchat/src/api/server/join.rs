use std::sync::Arc;

use tokio::sync::Mutex;

use serde_json::json;

use hyperborealib::http::HttpClient;
use hyperborealib::rest_api::prelude::*;

use crate::params::Params;

use super::process;

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