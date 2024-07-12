use hyperelm::exports::hyperborealib;
use hyperelm::client::ClientApp;

use hyperborealib::crypto::*;
use hyperborealib::rest_api::prelude::*;

pub mod params;
pub mod client;
pub mod server;
pub mod chat_ui;

use client::*;
use server::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = std::env::args()
        .collect::<Vec<_>>();

    let params = params::read().await?;

    match args.get(1).map(String::as_str) {
        Some("client") => {
            let Some(chat_server) = args.get(2) else {
                eprintln!("No chat server public key provided");

                return Ok(());
            };

            let chat_server = PublicKey::from_base64(chat_server)?;

            let server_app = ServerApp::from_params(params.clone());
            let member_app = ChatMemberApp::from_params(&params)?;

            // Run local server in background
            println!("Starting local server...");

            let server = tokio::spawn(async move {
                if let Err(err) = hyperelm::server::run(server_app).await {
                    log::error!("Server closed: {err:?}");
                };
            });

            // Wait a little before connecting to the server
            // so it's ready to handle incoming requests
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;

            // Run chat member client
            println!("Starting chat client app...");

            let client = hyperelm::client::run(member_app).await
                .map_err(|err| anyhow::anyhow!("Failed to connect to the local server: {err}"))?;

            // Lookup chat room server through the network
            println!("Looking up given chat room server...");

            loop {
                // Try to lookup chat room server
                // This method returns error only if initial lookup
                // request failed, which means we failed to request
                // our own local server - which is definitely bad
                let result = client.lookup(chat_server.clone(), Some(ClientType::Thin)).await
                    .map_err(|err| anyhow::anyhow!(err.to_string()))?;

                if let Some(chat_hoster) = result {
                    println!("Chat room found");
                    println!("  Client public key : {}", chat_hoster.client_public.to_base64());
                    println!("  Server address    : {}", &chat_hoster.server_address);

                    // Start chat UI
                    chat_ui::run(client, chat_hoster, params).await?;

                    // Stop chat room lookup loop
                    break;
                }

                // Timeout next request
                println!("Server not found. Trying again in {} seconds...", params.room_lookup_delay);

                tokio::time::sleep(std::time::Duration::from_secs(params.room_lookup_delay)).await;
            }

            // Stop the server
            server.abort();
        }

        Some("server") => {
            // Start logger
            env_logger::builder()
                .default_format()
                .filter_level({
                    if cfg!(debug_assertions) {
                        log::LevelFilter::max()
                    } else {
                        log::LevelFilter::Info
                    }
                })
                .parse_default_env()
                .init();

            // Print info about the chat room
            let server_secret = SecretKey::from_base64(&params.server_secret)?;
            let room_secret = SecretKey::from_base64(&params.room_secret_key)?;

            log::info!("Starting chat room server");
            log::info!("  Server public key : {}", server_secret.public_key().to_base64());
            log::info!("  Room public key   : {}", room_secret.public_key().to_base64());
            log::info!("  Local address     : {}", &params.server_local_address);
            log::info!("  Exposed address   : {}", &params.server_exposed_address);
            log::info!("  Room name         : {}", &params.room_name);
            log::info!("");

            // Start local HTTP REST API server and chat room hoster client
            let server_app = ServerApp::from_params(params.clone());
            let hoster_app = ChatHosterApp::from_params(&params)?;

            let server = tokio::spawn(async move {
                if let Err(err) = hyperelm::server::run(server_app).await {
                    log::error!("Server closed: {err:?}");
                };
            });

            // Wait a little before connecting to the server
            // so it's ready to handle incoming requests
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;

            hyperelm::client::run(hoster_app).await?;

            // Await until the server is closed (crashed)
            server.await?;
        }

        Some(command) => eprintln!("Unknown command: {command}"),
        None => eprintln!("No command provided")
    }

    Ok(())
}
