use tokio::sync::mpsc::{channel, unbounded_channel};

use serde_json::{json, Value as Json};

use hyperborealib::crypto::PublicKey;
use hyperborealib::http::HttpClient;
use hyperborealib::rest_api::prelude::*;

use crate::params::Params;

use super::*;

pub async fn request_join<T: HttpClient>(
    client: &ClientMiddleware<T>,
    params: &Params,
    server_address: &str,
    client_public: &PublicKey,
    username: &str
) -> anyhow::Result<Vec<(Sender, String)>> {
    let body = json!({
        "username": username
    });

    let (send, mut recv) = channel(1);

    let process = |response: Json, _info| async move {
        let users = serde_json::from_value(response["users"].clone())?;

        send.send(users).await?;

        Ok(())
    };

    request(
        client,
        params,
        server_address,
        client_public,
        "hyperchat/requests/join",
        &body,
        process
    ).await?;

    Ok(recv.recv().await.unwrap())
}

pub async fn listen_join<T: HttpClient>(
    client: &ClientMiddleware<T>,
    params: &Params
) -> anyhow::Result<Vec<(Sender, String)>> {
    let (send, mut recv) = unbounded_channel();

    listen(client, params, "hyperchat/announcements/join", |message, _info| async move {
        let client = serde_json::from_value(message["client"].clone())?;
        let server = serde_json::from_value(message["server"].clone())?;
        let username = message["username"].as_str().unwrap().to_string();

        send.send((
            Sender::new(client, server),
            username
        ))?;

        Ok(())
    }).await?;

    let mut joined = Vec::new();

    while let Some(member) = recv.recv().await {
        joined.push(member);
    }

    Ok(joined)
}
