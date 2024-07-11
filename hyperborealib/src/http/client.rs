use serde_json::Value as Json;

use crate::rest_api::AsJson;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Response {
    pub status: u16,
    pub body: Option<Json>
}

#[async_trait::async_trait]
pub trait HttpClient: Clone + Send + Sync {
    /// Send HTTP GET request
    async fn get(&self, url: impl AsRef<str> + Send) -> Result<Response, Box<dyn std::error::Error + Send + Sync>>;

    /// Send HTTP POST request with JSON body
    async fn post(&self, url: impl AsRef<str> + Send, body: Json) -> Result<Response, Box<dyn std::error::Error + Send + Sync>>;

    /// Perform GET REST API request
    async fn get_request<T: AsJson>(&self, url: impl AsRef<str> + Send) -> Result<T, Box<dyn std::error::Error + Send + Sync>> {
        #[cfg(feature = "tracing")]
        tracing::trace!(
            url = url.as_ref(),
            response_type = std::any::type_name::<T>(),
            "Performing HTTP GET request"
        );

        let response = self.get(url).await?;

        let Some(body) = response.body else {
            #[cfg(feature = "tracing")]
            tracing::error!("Request failed: no response body found");

            return Err("No response body found".into());
        };

        #[cfg(feature = "tracing")]
        tracing::trace!(
            response = ?body,
            "Received response"
        );

        Ok(T::from_json(&body)?)
    }

    /// Perform POST REST API request
    async fn post_request<T: AsJson + Send, F: AsJson>(&self, url: impl AsRef<str> + Send, request: T) -> Result<F, Box<dyn std::error::Error + Send + Sync>> {
        let request = request.to_json()?;

        #[cfg(feature = "tracing")]
        tracing::trace!(
            url = url.as_ref(),
            request_type = std::any::type_name::<T>(),
            response_type = std::any::type_name::<F>(),
            request_body = ?request,
            "Performing HTTP POST request"
        );

        let response = self.post(url, request).await?;

        let Some(body) = response.body else {
            #[cfg(feature = "tracing")]
            tracing::error!("Request failed: no response body found");

            return Err("No response body found".into());
        };

        #[cfg(feature = "tracing")]
        tracing::trace!(
            response = ?body,
            "Received response"
        );

        Ok(F::from_json(&body)?)
    }
}

#[cfg(feature = "client-reqwest")]
#[derive(Debug, Clone)]
pub struct ReqwestHttpClient(reqwest::Client);

#[cfg(feature = "client-reqwest")]
impl Default for ReqwestHttpClient {
    #[inline]
    fn default() -> Self {
        Self(reqwest::Client::new())
    }
}

#[cfg(feature = "client-reqwest")]
#[async_trait::async_trait]
impl HttpClient for ReqwestHttpClient {
    async fn get(&self, url: impl AsRef<str> + Send) -> Result<Response, Box<dyn std::error::Error + Send + Sync>> {
        let response = self.0.get(url.as_ref())
            .send().await
            .map_err(Box::new)?;

        let status = response.status();

        let body = response.json::<Json>().await
            .map_err(Box::new)?;

        Ok(Response {
            status: status.as_u16(),
            body: Some(body)
        })
    }

    async fn post(&self, url: impl AsRef<str> + Send, body: Json) -> Result<Response, Box<dyn std::error::Error + Send + Sync>> {
        let response = self.0.post(url.as_ref())
            .json(&body)
            .send().await
            .map_err(Box::new)?;

        let status = response.status();

        let body = response.json::<Json>().await
            .map_err(Box::new)?;

        Ok(Response {
            status: status.as_u16(),
            body: Some(body)
        })
    }
}
