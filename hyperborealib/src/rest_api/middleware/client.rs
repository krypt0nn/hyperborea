use std::collections::{HashSet, VecDeque};

use crate::crypto::asymmetric::PublicKey;
use crate::http::client::HttpClient;
use crate::drivers::ClientDriver;

use crate::rest_api::prelude::{
    *,
    Client as ClientApiRecord,
    Server as ServerApiRecord
};

use super::Error;

#[derive(Debug, Clone, Hash)]
/// Client HTTP middleware
/// 
/// This struct is used to perform HTTP REST API requests
/// to the servers from the name of inner client driver.
pub struct Client<T> {
    http_client: T,
    driver: ClientDriver
}

impl<T: HttpClient + Send + Sync> Client<T> {
    #[inline]
    pub fn new(http_client: T, client_driver: ClientDriver) -> Self {
        #[cfg(feature = "tracing")]
        tracing::trace!(
            http_client_type = std::any::type_name::<T>(),
            client_secret = client_driver.secret_key().to_base64(),
            client_info = ?client_driver.info(),
            "Building client REST API middleware"
        );

        Self {
            http_client,
            driver: client_driver
        }
    }

    #[inline]
    pub fn http_client(&self) -> &T {
        &self.http_client
    }

    #[inline]
    pub fn driver(&self) -> &ClientDriver {
        &self.driver
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(ret, skip_all, fields(
        server_address = server_address.as_ref()
    )))]
    /// Request server's info and validate its cryptographic correctness.
    /// 
    /// This method will perform `GET /api/v1/info` request.
    pub async fn get_info(&self, server_address: impl AsRef<str>) -> Result<InfoResponse, Error> {
        #[cfg(feature = "tracing")]
        tracing::debug!(
            server_address = server_address.as_ref(),
            "Sending GET /api/v1/info"
        );

        let response = self.http_client.get_request::<InfoResponse>(
            format!("http://{}/api/v1/info", server_address.as_ref())
        ).await?;

        if response.proof_seed < 1 << 63 {
            return Err(Error::InvalidProofSeed);
        }

        if !response.public_key.verify_signature(response.proof_seed.to_be_bytes(), &response.proof_sign)? {
            return Err(Error::InvalidProofSeedSignature);
        }

        Ok(response)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(ret, skip_all, fields(
        server_address = server_address.as_ref()
    )))]
    /// Request list of local server's clients
    /// 
    /// This method will perform `GET /api/v1/clients` request.
    pub async fn get_clients(&self, server_address: impl AsRef<str>) -> Result<Vec<ClientApiRecord>, Error> {
        #[cfg(feature = "tracing")]
        tracing::debug!(
            server_address = server_address.as_ref(),
            "Sending GET /api/v1/clients"
        );

        let response = self.http_client.get_request::<ClientsResponse>(
            format!("http://{}/api/v1/clients", server_address.as_ref())
        ).await?;

        Ok(response.clients)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(ret, skip_all, fields(
        server_address = server_address.as_ref()
    )))]
    /// Request list of servers known to given server
    /// 
    /// This method will perform `GET /api/v1/servers` request.
    pub async fn get_servers(&self, server_address: impl AsRef<str>) -> Result<Vec<ServerApiRecord>, Error> {
        #[cfg(feature = "tracing")]
        tracing::debug!(
            server_address = server_address.as_ref(),
            "Sending GET /api/v1/servers"
        );

        let response = self.http_client.get_request::<ServersResponse>(
            format!("http://{}/api/v1/servers", server_address.as_ref())
        ).await?;

        Ok(response.servers)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(ret, skip_all, fields(
        server_address = server_address.as_ref()
    )))]
    /// Connect to the server
    /// 
    /// This method will call `get_info` method to request
    /// the server's public key and then call `connect_to` method.
    pub async fn connect(&self, server_address: impl AsRef<str>) -> Result<(), Error> {
        let server_info = self.get_info(server_address.as_ref()).await?;

        self.connect_to(server_address, server_info.public_key).await
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(ret, skip_all, fields(
        server_address = server_address.as_ref(),
        server_public = server_public.to_base64()
    )))]
    /// Connect to the server with expected public key
    /// 
    /// This method will perform `POST /api/v1/connect` request.
    /// 
    /// In this method we expect that the given server has
    /// given public key. We need it to create connection
    /// certificate.
    pub async fn connect_to(&self, server_address: impl AsRef<str>, server_public: PublicKey) -> Result<(), Error> {
        let request = ConnectRequest::new(
            self.driver.secret_key(),
            server_public,
            self.driver.info().clone()
        );

        let proof_seed = request.0.proof_seed;

        #[cfg(feature = "tracing")]
        tracing::debug!(
            server_address = server_address.as_ref(),
            client_public = request.0.public_key.to_base64(),
            "Sending POST /api/v1/connect"
        );

        let response = self.http_client.post_request::<ConnectRequest, ConnectResponse>(
            format!("http://{}/api/v1/connect", server_address.as_ref()),
            request
        ).await?;

        if !response.validate(proof_seed)? {
            return Err(Error::InvalidProofSeedSignature);
        }

        if let Response::Error { status, reason, .. } = response.0 {
            return Err(Error::RequestFailed {
                status,
                reason
            });
        }

        Ok(())
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(ret, skip_all, fields(
        server_address = server_address.to_string(),
        client_public = client_public.to_base64(),
        client_type = ?client_type
    )))]
    /// Lookup client using given server
    /// 
    /// This method will perform `POST /api/v1/lookup` request.
    /// 
    /// `client_type` field is used as an optional filter.
    /// 
    /// This method will keep requesting servers until no more
    /// hints returned or needed client is found.
    pub async fn lookup(&self, server_address: impl ToString, client_public: PublicKey, client_type: Option<ClientType>) -> Result<Option<(ClientApiRecord, ServerApiRecord, bool)>, Error> {
        let request = LookupRequest::new(self.driver.secret_key(), client_public, client_type);

        let proof_seed = request.0.proof_seed;

        let mut queue = VecDeque::from([
            server_address.to_string()
        ]);

        let mut used_servers = HashSet::new();

        while let Some(server_address) = queue.pop_front() {
            // Skip server if it was already used
            if used_servers.contains(&server_address) {
                continue;
            }

            #[cfg(feature = "tracing")]
            tracing::debug!(
                server_address,
                client_public = request.0.public_key.to_base64(),
                "Sending POST /api/v1/lookup"
            );

            // Send lookup request
            let response = self.http_client.post_request::<LookupRequest, LookupResponse>(
                format!("http://{server_address}/api/v1/lookup"),
                request.clone()
            ).await?;

            // Validate response
            if !response.validate(proof_seed)? {
                // Skip execution and go to the next server
                continue;
            }

            // Process successful response
            if let Response::Success { public_key, response, .. } = response.0 {
                match response {
                    LookupResponseBody::Local { client, available } => {
                        let server = ServerApiRecord::new(public_key, &server_address);

                        return Ok(Some((client, server, available)));
                    }

                    LookupResponseBody::Remote { client, server, available } => {
                        return Ok(Some((client, server, available)));
                    }

                    LookupResponseBody::Hint { mut servers } => {
                        for server in servers.drain(..) {
                            queue.push_back(server.address);
                        }
                    }
                }
            }

            used_servers.insert(server_address);
        }

        Ok(None)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(ret, skip_all, fields(
        server_address = server_address.as_ref(),
        client_public = client_public.to_base64(),
        sender = ?sender,
        channel = channel.to_string(),
        message = ?message
    )))]
    /// Send message to the given client
    /// 
    /// This method will perform `POST /api/v1/send` request.
    pub async fn send(&self, server_address: impl AsRef<str>, client_public: PublicKey, sender: Sender, channel: impl ToString, message: Message) -> Result<(), Error> {
        let request = SendRequest::new(self.driver.secret_key(), sender, client_public, channel, message);

        let proof_seed = request.0.proof_seed;

        #[cfg(feature = "tracing")]
        tracing::debug!(
            server_address = server_address.as_ref(),
            client_public = request.0.public_key.to_base64(),
            "Sending POST /api/v1/send"
        );

        // Send request
        let response = self.http_client.post_request::<SendRequest, SendResponse>(
            format!("http://{}/api/v1/send", server_address.as_ref()),
            request.clone()
        ).await?;

        // Validate response
        if !response.validate(proof_seed)? {
            return Err(Error::InvalidProofSeedSignature);
        }

        // Check response status
        if let Response::Error { status, reason, .. } = response.0 {
            return Err(Error::RequestFailed {
                status,
                reason
            });
        }

        Ok(())
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(ret, skip_all, fields(
        server_address = server_address.as_ref(),
        channel = channel.to_string(),
        limit
    )))]
    /// Poll messages sent to the given client
    /// 
    /// This method will perform `POST /api/v1/poll` request.
    /// 
    /// Returns list of messages and amount of remained.
    pub async fn poll(&self, server_address: impl AsRef<str>, channel: impl ToString, limit: Option<u64>) -> Result<(Vec<MessageInfo>, u64), Error> {
        let request = PollRequest::new(self.driver.secret_key(), channel, limit);

        let proof_seed = request.0.proof_seed;

        #[cfg(feature = "tracing")]
        tracing::debug!(
            server_address = server_address.as_ref(),
            client_public = request.0.public_key.to_base64(),
            "Sending POST /api/v1/poll"
        );

        // Send poll request
        let response = self.http_client.post_request::<PollRequest, PollResponse>(
            format!("http://{}/api/v1/poll", server_address.as_ref()),
            request.clone()
        ).await?;

        // Validate response
        if !response.validate(proof_seed)? {
            return Err(Error::InvalidProofSeedSignature);
        }

        // Process response
        match response.0 {
            Response::Success { response, .. } => {
                Ok((
                    response.messages,
                    response.remaining
                ))
            }

            Response::Error { status, reason, .. } => {
                Err(Error::RequestFailed {
                    status,
                    reason
                })
            }
        }
    }
}
