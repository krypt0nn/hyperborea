use hyperborealib::crypto::*;
use hyperborealib::rest_api::prelude::*;

pub mod params;
pub mod client;
pub mod server;
pub mod chat_server;
pub mod chat_ui;

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

            // Create client middlewire
            let client = client::new(&params).await?;

            // Run local server in background
            println!("Starting local server...");

            let server = {
                let params = params.clone();

                tokio::spawn(async move {
                    server::run(&params).await
                })
            };

            // Wait a little before connecting to the server
            // so it's ready to handle incoming requests
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;

            // Connect to the local server
            println!("Connecting to the local server...");

            client.connect(&params.server_local_address).await
                .map_err(|err| anyhow::anyhow!(err.to_string()))?;

            // Lookup chat room server through the network
            println!("Looking up given chat room server...");

            loop {
                // Try to lookup chat room server
                // This method returns error only if initial lookup
                // request failed, which means we failed to request
                // our own local server - which is definitely bad
                let result = client.lookup(
                    &params.server_local_address,
                    chat_server.clone(),
                    Some(ClientType::Server)
                ).await.map_err(|err| anyhow::anyhow!(err.to_string()))?;

                if let Some((chat_client, chat_server, _available)) = result {
                    println!("Chat room server found");
                    println!("  Client public key : {}", chat_client.public_key.to_base64());
                    println!("  Server public key : {}", chat_server.public_key.to_base64());
                    println!("  Server address    : {}", &chat_server.address);

                    // Start chat UI
                    chat_ui::run().await?;

                    // Stop server when the UI is closed
                    server.abort();

                    // Stop chat room lookup loop
                    break;
                }

                // Timeout next request
                println!("Server not found. Trying again in {} seconds...", params.room_lookup_delay);

                tokio::time::sleep(std::time::Duration::from_secs(params.room_lookup_delay)).await;
            }
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

            // Print info about the server
            log::info!("Starting chat room server");
            log::info!("  Public key      : {}", SecretKey::from_base64(&params.server_secret)?.public_key().to_base64());
            log::info!("  Local address   : {}", &params.server_local_address);
            log::info!("  Exposed address : {}", &params.server_exposed_address);
            log::info!("");

            // Start local HTTP REST API server
            server::run(&params).await?;
        }

        Some(command) => eprintln!("Unknown command: {command}"),
        None => eprintln!("No command provided")
    }

    Ok(())
}
