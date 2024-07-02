use hyperborealib::crypto::SecretKey;

use hyperborealib::http::{
    ReqwestHttpClient,
    AxumHttpServer
};

use hyperborealib::drivers::ServerDriver;
use hyperborealib::drivers::server::prelude::*;

use hyperborealib::rest_api::prelude::*;

use crate::params::Params;

pub async fn run(params: &Params) -> anyhow::Result<()> {
    let driver =ServerDriver::new(
        GlobalTableRouter::default(),
        BfsRecursionTraversal,
        BasicInbox::default(),
        ServerParams {
            server_secret: SecretKey::from_base64(&params.server_secret)?,
            server_address: params.server_exposed_address.clone()
        }
    );

    let server = ServerMiddleware::new(
        ReqwestHttpClient::default(),
        AxumHttpServer::default(),
        driver
    ).await;

    // Run background network traversal process
    {
        let params = params.clone();
        let driver = server.driver();

        tokio::spawn(async move {
            let server_client = ClientMiddleware::new(
                ReqwestHttpClient::default(),
                driver.as_client()
            );

            loop {
                for address in params.bootstrap_addresses.clone() {
                    if let Ok(info) = server_client.get_info(&address).await {
                        // FIXME
                        // driver.router().index_server(Server::new(
                        //     info.public_key,
                        //     address
                        // )).await;
                    }
                }

                let http = ReqwestHttpClient::default();

                driver.traversal().traverse(http, &driver).await;

                tokio::time::sleep(std::time::Duration::from_secs(params.bootstrap_traversal_delay)).await;
            }
        });
    }

    // Start HTTP REST API server
    server.serve(&params.server_local_address).await
        .map_err(|err| anyhow::anyhow!(err.to_string()))?;

    Ok(())
}
