use hyperborealib::http::client::HttpClient;
use hyperborealib::rest_api::middleware::Client as ClientMiddleware;

pub async fn command_servers<T: HttpClient>(middleware: &ClientMiddleware<T>, address: impl std::fmt::Display) {
    match middleware.get_servers(address).await {
        Ok(servers) => {
            log::info!("");
            log::info!("Received {} servers", servers.len());

            if !servers.is_empty() {
                log::info!("");
            }

            for server in servers {
                log::info!("  [{:24}] {}", server.address, server.public_key.to_base64())
            }
        }

        Err(err) => {
            log::error!("");
            log::error!("Failed to get info from the server: {err}");
        }
    }
}
