use hyperborealib::crypto::SecretKey;
use hyperborealib::http::ReqwestHttpClient;
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
        &client.driver().secret_key(),
        server_secret.public_key()
    );

    // Prepare chat endpoint info for incoming requests
    let chat_sender = Sender::new(
        Client::new(client.driver().secret_key().public_key(), certificate, ClientInfo::thin()),
        Server::new(server_secret.public_key(), params.server_exposed_address.clone())
    );

    let mut users = Vec::<(Sender, String)>::new();

    // Start the chat room server
    loop {
        // Poll join requests
        let (joins, _) = client.poll(&params.server_local_address, "hyperchat/join", None).await
            .map_err(|err| anyhow::anyhow!(err.to_string()))?;

        for join in joins {
            // Read join message
            let message = join.message.read(
                client.driver().secret_key(),
                &join.sender.client.public_key
            ).map_err(|err| anyhow::anyhow!(err.to_string()))?;

            // Filter username
            let username = String::from_utf8_lossy(&message)
                .chars()
                .filter(|c| !c.is_control())
                .collect::<String>();

            // Announce other users about newcomer
            log::info!("[join] {username} joins room");

            for (user, _) in &users {
                let message = Message::create(
                    client.driver().secret_key(),
                    &user.client.public_key,
                    serde_json::to_string(&users),
                    encoding
                );

                client.send(
                    &user.server.address,
                    user.client.public_key.clone(),
                    chat_sender.clone(),
                    "hyperchat/join",
                    message
                );
            }

            // Store the info about the new user
            users.push((
                join.sender,
                username
            ));
        }

        // Timeout til next update
        tokio::time::sleep(std::time::Duration::from_millis(params.room_sync_delay)).await;
    }

    Ok(())
}
