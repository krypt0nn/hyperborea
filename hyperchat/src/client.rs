use hyperborealib::crypto::SecretKey;
use hyperborealib::http::ReqwestHttpClient;
use hyperborealib::drivers::ClientDriver;
use hyperborealib::rest_api::prelude::*;

use crate::params::Params;

pub async fn new(params: &Params) -> anyhow::Result<ClientMiddleware<ReqwestHttpClient>> {
    let secret = SecretKey::from_base64(&params.client_secret)?;

    let driver = ClientDriver::thin(secret);

    let client = ClientMiddleware::new(ReqwestHttpClient::default(), driver);

    Ok(client)
}
