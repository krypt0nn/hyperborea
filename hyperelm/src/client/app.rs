use serde_json::{json, Value as Json};

use hyperborealib::exports::tokio;
use hyperborealib::crypto::*;
use hyperborealib::http::HttpClient;
use hyperborealib::rest_api::prelude::*;

use super::*;

#[derive(Debug, thiserror::Error)]
pub enum ClientAppError<E: Send + Sync> {
    #[error(transparent)]
    SerdeJsonError(#[from] serde_json::Error),

    #[error(transparent)]
    AsJsonError(#[from] AsJsonError),

    #[error(transparent)]
    MiddlewareError(#[from] MiddlewareError),

    #[error(transparent)]
    SendError(#[from] SendError),

    #[error(transparent)]
    Custom(E)
}

#[async_trait::async_trait]
pub trait ClientApp {
    /// Request which can be received from other clients.
    type InputRequest: AsJson + Send;

    /// Response which will be sent after processing the `InputRequest`.
    type InputResponse: AsJson + Send;

    /// Message which can be received from other clients.
    type InputMessage: AsJson + Send;

    /// Request which can be sent to other clients.
    type OutputRequest: AsJson + Send;

    /// Response which can be received from other clients.
    type OutputResponse: AsJson + Send;

    /// Message which can be sent to other clients.
    type OutputMessage: AsJson + Send;

    type HttpClient: HttpClient;
    type State;
    type Error: Send + Sync;

    fn get_params(&self) -> &ClientAppParams;
    fn get_middlewire(&self) -> &ClientMiddleware<Self::HttpClient>;
    fn get_state(&self) -> Arc<Self::State>;

    /// Get certificate that proves that the client is connected
    /// to the given server.
    fn get_certificate(&self) -> ConnectionCertificate {
        let params = self.get_params();

        ConnectionCertificate::new(
            &params.client_secret,
            params.server_public.clone()
        )
    }

    /// Get sender information about the current client.
    fn get_sender(&self) -> Sender {
        let params = self.get_params();

        let client = Client::new(
            params.client_secret.public_key(),
            self.get_certificate(),
            ClientInfo::thin()
        );

        let server = Server::new(
            params.server_public.clone(),
            &params.server_address
        );

        Sender::new(client, server)
    }

    /// Perform client searching in the network.
    async fn lookup(&self, public_key: PublicKey, client_type: Option<ClientType>) -> Result<Option<ClientEndpoint>, ClientAppError<Self::Error>> {
        let server_address = &self.get_params().server_address;

        let result = self.get_middlewire()
            .lookup(server_address, public_key, client_type).await?
            .map(|(client, server, _)| {
                ClientEndpoint {
                    server_address: server.address,
                    client_public: client.public_key
                }
            });

        Ok(result)
    }

    /// Send request to given endpoint.
    async fn request(&self, endpoint: ClientEndpoint, request: Self::OutputRequest) -> Result<Self::OutputResponse, ClientAppError<Self::Error>> {
        let params = self.get_params();
        let middlewire = self.get_middlewire();

        // Prepare request
        let request_id = safe_random_u64();

        let request = json!({
            "id": request_id,
            "request": request.to_json()?
        });

        // Send request
        let request = Message::create(
            &params.client_secret,
            &endpoint.client_public,
            serde_json::to_vec(&request)?,
            params.encoding
        )?;

        middlewire.send(
            &endpoint.server_address,
            endpoint.client_public,
            self.get_sender(),
            &params.channel,
            request
        ).await?;

        // Receive response
        loop {
            let (messages, _) = middlewire.poll(
                &endpoint.server_address,
                format!("{}@{request_id}", params.channel),
                Some(1)
            ).await?;

            // If there's an incoming message
            if let Some(message) = messages.first() {
                // Decode the message and verify its validity
                let response = message.message.read(
                    &params.client_secret,
                    &message.sender.client.public_key
                )?;

                // Deserialize it and return
                let response = serde_json::from_slice::<Json>(&response)?;

                let response = Self::OutputResponse::from_json(&response)?;

                return Ok(response);
            }

            // Sleep otherwise and try again
            tokio::time::sleep(params.delay).await;
        }
    }

    /// Send message to given endpoint.
    async fn send(&self, endpoint: ClientEndpoint, message: Self::OutputMessage) -> Result<(), ClientAppError<Self::Error>> {
        let params = self.get_params();
        let middlewire = self.get_middlewire();

        // Prepare message
        let message = json!({
            "message": message.to_json()?
        });

        let message = Message::create(
            &params.client_secret,
            &endpoint.client_public,
            serde_json::to_vec(&message)?,
            params.encoding
        )?;

        // Send message
        middlewire.send(
            endpoint.server_address,
            endpoint.client_public,
            self.get_sender(),
            &params.channel,
            message
        ).await?;

        Ok(())
    }

    /// Receive and process incoming messages.
    async fn update(&self) -> Result<(), ClientAppError<Self::Error>> {
        let params = self.get_params();
        let middlewire = self.get_middlewire();

        let (messages, _) = middlewire.poll(&params.server_address, &params.channel, None).await?;

        for message_info in messages {
            // Decode the message and verify its validity
            let content = message_info.message.read(
                &params.client_secret,
                &message_info.sender.client.public_key
            )?;

            // Deserialize it and process
            let content = serde_json::from_slice::<Json>(&content)?;

            // Handle request
            if let Some(request) = content.get("request") {
                if let Some(request_id) = content.get("id").and_then(Json::as_u64) {
                    // Deserialize request
                    let request = Self::InputRequest::from_json(request)?;

                    // Process request
                    let response = self.handle_request(request, message_info.clone()).await?;

                    // Send response
                    let response = Message::create(
                        &params.client_secret,
                        &message_info.sender.client.public_key,
                        serde_json::to_vec(&response.to_json()?)?,
                        params.encoding
                    )?;

                    middlewire.send(
                        &message_info.sender.server.address,
                        message_info.sender.client.public_key,
                        self.get_sender(),
                        format!("{}@{request_id}", params.channel),
                        response
                    ).await?;
                }
            }

            // Handle message
            else if let Some(message) = content.get("message") {
                let message = Self::InputMessage::from_json(message)?;

                // Process message
                self.handle_message(message, message_info).await?;
            }
        }

        Ok(())
    }

    /// Handle incoming request.
    async fn handle_request(&self, request: Self::InputRequest, info: MessageInfo) -> Result<Self::InputResponse, ClientAppError<Self::Error>>;

    /// Handle incoming message.
    async fn handle_message(&self, message: Self::InputMessage, info: MessageInfo) -> Result<(), ClientAppError<Self::Error>>;
}
