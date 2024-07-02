use std::sync::Arc;

use tokio::sync::Mutex;

use hyperborealib::crypto::SecretKey;
use hyperborealib::http::ReqwestHttpClient;
use hyperborealib::drivers::ClientDriver;
use hyperborealib::rest_api::prelude::*;

use crate::params::Params;

mod process;
mod join;

pub use process::process;
pub use join::process_joins;

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
