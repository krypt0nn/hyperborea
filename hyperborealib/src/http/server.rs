use std::net::{
    SocketAddr,
    ToSocketAddrs
};

#[cfg(feature = "server-axum")]
use tokio::net::TcpListener;

#[cfg(feature = "server-axum")]
use axum::{
    extract::ConnectInfo,
    body::Bytes as HttpBody
};

use crate::rest_api::AsJson;

#[async_trait::async_trait]
pub trait HttpServer {
    /// Add GET request route
    async fn get<T: AsJson, F: std::future::Future<Output = T> + Send>(
        &mut self,
        path: impl AsRef<str> + Send,
        callback: impl FnOnce(SocketAddr) -> F + Clone + Send + Sync + 'static
    );

    /// Add POST request route
    async fn post<T: AsJson, F: AsJson, R: std::future::Future<Output = F> + Send>(
        &mut self,
        path: impl AsRef<str> + Send,
        callback: impl FnOnce(SocketAddr, T) -> R + Clone + Send + Sync + 'static
    );

    /// Run the server with specified GET and POST routes
    async fn serve(self, address: impl ToSocketAddrs + Send) -> Result<(), Box<dyn std::error::Error>>;
}

#[cfg(feature = "server-axum")]
#[derive(Default, Debug, Clone)]
pub struct AxumHttpServer(Option<axum::Router>);

#[cfg(feature = "server-axum")]
#[async_trait::async_trait]
impl HttpServer for AxumHttpServer {
    async fn get<T: AsJson, F: std::future::Future<Output = T> + Send>(
        &mut self,
        path: impl AsRef<str> + Send,
        callback: impl FnOnce(SocketAddr) -> F + Clone + Send + Sync + 'static
    ) {
        let router = self.0.take().unwrap_or_default();

        self.0 = Some(router.route(path.as_ref(), axum::routing::get(move |ConnectInfo(client_address): ConnectInfo<SocketAddr>| async move {
            let response = callback(client_address).await;

            match response.to_json() {
                Ok(response) => {
                    axum::http::Response::builder()
                        .header("Content-Type", "text/json")
                        .body(response.to_string())
                        .unwrap()
                }

                Err(err) => {
                    axum::http::Response::builder()
                        .status(500)
                        .body(format!("Failed to serialize response as JSON: {err}"))
                        .unwrap()
                }
            }
        })));
    }

    async fn post<T: AsJson, F: AsJson, R: std::future::Future<Output = F> + Send>(
        &mut self,
        path: impl AsRef<str> + Send,
        callback: impl FnOnce(SocketAddr, T) -> R + Clone + Send + Sync + 'static
    ) {
        let router = self.0.take().unwrap_or_default();

        self.0 = Some(router.route(path.as_ref(), axum::routing::post(move |ConnectInfo(client_address): ConnectInfo<SocketAddr>, body: HttpBody| async move {
            let json = match serde_json::from_slice::<serde_json::Value>(&body) {
                Ok(json) => json,
                Err(err) => {
                    return axum::http::Response::builder()
                        .status(500)
                        .body(format!("Failed to deserialize request JSON: {err}"))
                        .unwrap();
                }
            };

            let request = match T::from_json(&json) {
                Ok(request) => request,
                Err(err) => {
                    return axum::http::Response::builder()
                        .status(500)
                        .body(format!("Failed to deserialize API request from JSON object: {err}"))
                        .unwrap();
                }
            };

            let response = callback(client_address, request).await;

            match response.to_json() {
                Ok(response) => {
                    axum::http::Response::builder()
                        .header("Content-Type", "text/json")
                        .body(response.to_string())
                        .unwrap()
                }

                Err(err) => {
                    axum::http::Response::builder()
                        .status(500)
                        .body(format!("Failed to serialize response as JSON: {err}"))
                        .unwrap()
                }
            }
        })));
    }

    async fn serve(mut self, address: impl ToSocketAddrs + Send) -> Result<(), Box<dyn std::error::Error>> {
        let router = self.0.take()
            .unwrap_or_default()
            .into_make_service_with_connect_info::<SocketAddr>();

        let Some(address) = address.to_socket_addrs()?.next() else {
            return Err("Failed to resolve server address".into());
        };

        let listener = TcpListener::bind(address).await?;

        axum::serve(listener, router).await?;

        Ok(())
    }
}
