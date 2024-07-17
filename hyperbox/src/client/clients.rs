use hyperborealib::http::client::HttpClient;
use hyperborealib::rest_api::middleware::Client as ClientMiddleware;

pub async fn command_clients<T: HttpClient>(middleware: &ClientMiddleware<T>, address: impl std::fmt::Display) {
    match middleware.get_clients(address).await {
        Ok(clients) => {
            log::info!("");
            log::info!("Received {} clients", clients.len());

            if !clients.is_empty() {
                log::info!("");
            }

            for client in clients {
                log::info!("  [{}] {:?}", client.public_key.to_base64(), client.info)
            }
        }

        Err(err) => {
            log::error!("");
            log::error!("Failed to get info from the server: {err}");
        }
    }
}
