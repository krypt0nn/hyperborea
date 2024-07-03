use client::ClientParams;

use hyperborealib::rest_api::prelude::*;

pub mod client;

pub mod prelude {
    pub use hyperborealib;

    pub use super::client::{
        ClientParams,
        ClientEndpoint,
        ClientApp,
        ClientAppError
    };

    pub use super::build_client;
}

pub mod exports {
    pub use hyperborealib;
    pub use tokio;
}

#[derive(serde::Serialize, serde::Deserialize)]
enum AppInputRequest {
    Sus,
    Banana
}

#[derive(serde::Serialize, serde::Deserialize)]
enum AppInputResponse {
    Amogus
}

impl hyperborealib::rest_api::AsJson for AppInputRequest {
    fn to_json(&self) -> Result<serde_json::Value, hyperborealib::rest_api::AsJsonError> {
        Ok(serde_json::to_value(self)?)
    }

    fn from_json(json: &serde_json::Value) -> Result<Self, hyperborealib::rest_api::AsJsonError> where Self: Sized {
        Ok(serde_json::from_value(json.clone())?)
    }
}

impl hyperborealib::rest_api::AsJson for AppInputResponse {
    fn to_json(&self) -> Result<serde_json::Value, hyperborealib::rest_api::AsJsonError> {
        Ok(serde_json::to_value(self)?)
    }

    fn from_json(json: &serde_json::Value) -> Result<Self, hyperborealib::rest_api::AsJsonError> where Self: Sized {
        Ok(serde_json::from_value(json.clone())?)
    }
}

struct App;

#[async_trait::async_trait]
impl client::ClientApp for App {
    build_client!(
        input: AppInputRequest => AppInputResponse, ();
        output: () => (), ();

        client: hyperborealib::http::ReqwestHttpClient;
 
        error: std::io::Error;

        requests: {
            AppInputRequest::Sus => async {
                Ok(AppInputResponse::Amogus)
            }

            AppInputRequest::Banana => async {
                Ok(AppInputResponse::Amogus)
            }
        };

        messages: {
            () => async {
                Ok(())
            }
        };
    );

    fn get_params(&self) ->  &ClientParams {
        todo!()
    }

    fn get_middlewire(&self) ->  &ClientMiddleware<Self::HttpClient>  {
        todo!()
    }
}
