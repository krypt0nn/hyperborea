use hyperborealib::prelude::*;

pub async fn command_lookup<T: HttpClient>(middleware: &ClientMiddleware<T>, address: impl ToString, client_public: PublicKey, client_type: Option<ClientType>) {
    match middleware.lookup(address, client_public, client_type).await {
        Ok(Some((client, server, available))) => {
            log::info!("");
            log::info!("Client found:");
            log::info!("  Public key        : {}", client.public_key.to_base64());
            log::info!("  Client type       : {}", client.info.client_type);
            log::info!("  Server public key : {}", server.public_key.to_base64());
            log::info!("  Server address    : {}", server.address);
            log::info!("  Is available?     : {}", if available { "Yes" } else { "No" });
        }

        Ok(None) => {
            log::info!("");
            log::info!("Requested client is not found");
        }

        Err(err) => {
            log::error!("");
            log::error!("Failed to lookup client: {err}");
        }
    }
}
