use hyperborealib::http::client::HttpClient;
use hyperborealib::rest_api::middleware::Client as ClientMiddleware;

pub async fn command_info<T: HttpClient>(middleware: &ClientMiddleware<T>, address: impl AsRef<str>) {
    match middleware.get_info(address).await {
        Ok(info) => {
            log::info!("");
            log::info!("Public key : {}", info.public_key.to_base64());
            log::info!("Standard   : v{}", info.standard);
        }

        Err(err) => {
            log::error!("");
            log::error!("Failed to get info from the server: {err}");
        }
    }
}
