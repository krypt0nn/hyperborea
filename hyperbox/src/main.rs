use hyperborealib::{LIBRARY_VERSION, STANDARD_VERSION};

use hyperborealib::http::client::ReqwestHttpClient;
use hyperborealib::http::server::AxumHttpServer;

use hyperborealib::crypto::PublicKey;

use hyperborealib::client::Client;

use hyperborealib::server::prelude::*;
use hyperborealib::rest_api::prelude::*;

pub mod args;
pub mod shell;
pub mod client;
pub mod server;

use args::Args;
use shell::Shell;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() -> anyhow::Result<()> {
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

    let args = Args::from_env();

    log::info!("Hyperbox v{VERSION}");
    log::info!("  Hyperborealib v{LIBRARY_VERSION}");
    log::info!("  Standard v{STANDARD_VERSION}");
    log::info!("");

    match args.command() {
        Some("client") => {
            let shell = Shell::new("client$ ", true);

            let client = Client::random();

            let middleware = ClientMiddleware::new(
                ReqwestHttpClient::default(),
                client
            );

            loop {
                let args = shell.poll()?;

                match args.command() {
                    Some("help") | None => {
                        log::info!("help              - show this help");
                        log::info!("exit              - close the shell");
                        log::info!("info <address>    - perform GET /api/v1/info request");
                        log::info!("clients <address> - perform GET /api/v1/clients request");
                        log::info!("servers <address> - perform GET /api/v1/servers request");
                        log::info!("lookup <address> <public> [<type>] - perform");
                        log::info!("                    POST /api/v1/lookup request");
                        log::info!("connect <address> - perform POST /api/v1/connect request,");
                        log::info!("                    opening new shell with related commands");
                    }

                    Some("exit") => break,

                    Some("info") => {
                        let Some(address) = args.args().first() else {
                            log::error!("No server address given");

                            continue;
                        };

                        client::command_info(&middleware, address).await;
                    }

                    Some("clients") => {
                        let Some(address) = args.args().first() else {
                            log::error!("No server address given");

                            continue;
                        };

                        client::command_clients(&middleware, address).await;
                    }

                    Some("servers") => {
                        let Some(address) = args.args().first() else {
                            log::error!("No server address given");

                            continue;
                        };

                        client::command_servers(&middleware, address).await;
                    }

                    Some("lookup") => {
                        let Some(address) = args.args().first() else {
                            log::error!("No server address given");

                            continue;
                        };

                        let Some(public_key) = args.args().get(1) else {
                            log::error!("No search client public key given");

                            continue;
                        };

                        let public_key = match PublicKey::from_base64(public_key) {
                            Ok(public_key) => public_key,
                            Err(err) => {
                                log::error!("Failed to deserialize public key from base64: {err}");

                                continue;
                            }
                        };

                        client::command_lookup(&middleware, address, public_key, None).await;
                    }

                    Some("connect") => {
                        let Some(address) = args.args().first() else {
                            log::error!("No server address given");

                            continue;
                        };

                        match middleware.connect(&address).await {
                            Ok(_) => {
                                log::info!("");
                                log::info!("Connected successfully");

                                let shell = Shell::new(format!("client({address})$ "), true);

                                loop {
                                    let args = shell.poll()?;

                                    match args.command() {
                                        Some("help") | None => {
                                            log::info!("help    - show this help");
                                            log::info!("exit    - close the shell");
                                            log::info!("info    - perform GET /api/v1/info request");
                                            log::info!("clients - perform GET /api/v1/clients request");
                                            log::info!("servers - perform GET /api/v1/servers request");
                                            log::info!("lookup <public> [<type>] - perform POST /api/v1/lookup request");
                                        }

                                        Some("exit") => break,

                                        Some("info") => client::command_info(&middleware, address).await,
                                        Some("clients") => client::command_clients(&middleware, address).await,
                                        Some("servers") => client::command_servers(&middleware, address).await,

                                        Some("lookup") => {
                                            let Some(public_key) = args.args().first() else {
                                                log::error!("No search client public key given");

                                                continue;
                                            };

                                            let public_key = match PublicKey::from_base64(public_key) {
                                                Ok(public_key) => public_key,
                                                Err(err) => {
                                                    log::error!("Failed to deserialize public key from base64: {err}");

                                                    continue;
                                                }
                                            };

                                            client::command_lookup(&middleware, address, public_key, None).await;
                                        }

                                        Some(command) => log::error!("Unknown command: {command}. Run help to get list of available commands")
                                    }
                                }
                            }

                            Err(err) => log::error!("Failed to get info from the server: {err}")
                        }
                    }

                    Some(command) => log::error!("Unknown command: {command}. Run help to get list of available commands")
                }
            }
        }

        Some("server") => {
            let shell = Shell::new("server$ ", true);

            loop {
                let args = shell.poll()?;

                match args.command() {
                    Some("help") | None => {
                        log::info!("help  - show this help");
                        log::info!("exit  - close the shell");
                        log::info!("start - start HTTP server");
                    }

                    Some("exit") => break,

                    Some("start") => {
                        let server = Server::new(
                            GlobalTableRouter::new(4096, std::time::Duration::from_secs(60 * 30)),
                            BfsRecursionTraversal,
                            Me
                            ServerParams::default()
                        );

                        let middleware = ServerMiddleware::new(
                            ReqwestHttpClient::default(),
                            AxumHttpServer::default(),
                            server
                        ).await;

                        if let Err(err) = middleware.serve("127.0.0.1:8001").await {
                            log::error!("HTTP server error: {err}");
                        }
                    }

                    Some(command) => log::error!("Unknown command: {command}. Run help to get list of available commands")
                }
            }
        }

        Some(command) => {
            log::error!("Unknown command: {}", command);
        }

        None => {
            log::error!("Missing command");
        }
    }

    Ok(())
}
