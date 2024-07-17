use std::sync::Arc;
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
    http_client: Arc<T>,
    driver: Arc<ClientDriver>
}

impl<T: HttpClient + Send + Sync> Client<T> {
    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all, fields(
        http_client_type = std::any::type_name::<T>(),
        client_secret = client_driver.secret_key().to_base64(),
        client_info = ?client_driver.info()
    )))]
    pub fn new(http_client: T, client_driver: ClientDriver) -> Self {
        #[cfg(feature = "tracing")]
        tracing::trace!("Building client REST API middleware");

        Self {
            http_client: Arc::new(http_client),
            driver: Arc::new(client_driver)
        }
    }

    #[inline]
    pub fn http_client(&self) -> Arc<T> {
        self.http_client.clone()
    }

    #[inline]
    pub fn http_client_ref(&self) -> &T {
        &self.http_client
    }

    #[inline]
    pub fn driver(&self) -> Arc<ClientDriver> {
        self.driver.clone()
    }

    #[inline]
    pub fn driver_ref(&self) -> &ClientDriver {
        &self.driver
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(ret, skip_all, fields(
        server_address
    )))]
    /// Request server info.
    /// 
    /// This method will perform `GET /api/v1/info` request.
    /// 
    /// - `server_address` must contain address of the server
    ///   from which we want to request the info.
    pub async fn get_info(&self, server_address: impl std::fmt::Display) -> Result<InfoResponse, Error> {
        #[cfg(feature = "tracing")]
        tracing::debug!("Sending GET /api/v1/info request");

        // Send get info request
        let response = self.http_client.get_request::<InfoResponse>(
            format!("http://{server_address}/api/v1/info")
        ).await?;

        // Validate response
        if !response.validate()? {
            return Err(Error::InvalidProofSeedSignature);
        }

        Ok(response)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(ret, skip_all, fields(
        server_address
    )))]
    /// Request list of local server's clients.
    /// 
    /// This method will perform `GET /api/v1/clients` request.
    /// 
    /// - `server_address` must contain address of the server
    ///   from which we want to request the clients list.
    pub async fn get_clients(&self, server_address: impl std::fmt::Display) -> Result<Vec<ClientApiRecord>, Error> {
        #[cfg(feature = "tracing")]
        tracing::debug!("Sending GET /api/v1/clients request");

        // Send get clients request
        let response = self.http_client.get_request::<ClientsResponse>(
            format!("http://{server_address}/api/v1/clients")
        ).await?;

        Ok(response.clients)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(ret, skip_all, fields(
        server_address
    )))]
    /// Request list of servers known to given server.
    /// 
    /// This method will perform `GET /api/v1/servers` request.
    /// 
    /// - `server_address` must contain address of the server
    ///   from which we want to request the servers list.
    pub async fn get_servers(&self, server_address: impl std::fmt::Display) -> Result<Vec<ServerApiRecord>, Error> {
        #[cfg(feature = "tracing")]
        tracing::debug!("Sending GET /api/v1/servers request");

        // Send get servers request
        let response = self.http_client.get_request::<ServersResponse>(
            format!("http://{server_address}/api/v1/servers")
        ).await?;

        Ok(response.servers)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all, fields(
        server_address
    )))]
    /// Connect to the server
    /// 
    /// This method will call `get_info` method to request
    /// the server's public key and then call `connect_to` method.
    /// 
    /// - `server_address` must contain address of the server
    ///   to which we want to connect.
    pub async fn connect(&self, server_address: impl std::fmt::Display + Clone) -> Result<ConnectedClient<T>, Error> {
        let server_info = self.get_info(server_address.clone()).await?;

        self.connect_to(server_address, server_info.public_key).await
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all, fields(
        server_address,
        server_public = server_public.to_base64()
    )))]
    /// Connect to the server with expected public key
    /// 
    /// This method will perform `POST /api/v1/connect` request.
    /// 
    /// In this method we expect that the given server has
    /// given public key. We need it to create connection
    /// certificate.
    pub async fn connect_to(&self, server_address: impl std::fmt::Display, server_public: PublicKey) -> Result<ConnectedClient<T>, Error> {
        #[cfg(feature = "tracing")]
        tracing::debug!("Sending POST /api/v1/connect request");

        // Prepare connect request
        let request = ConnectRequest::new(
            self.driver.secret_key(),
            server_public.clone(),
            self.driver.info().clone()
        );

        let proof_seed = request.0.proof_seed;
        let certificate = request.0.request.certificate.clone();

        // Send request
        let response = self.http_client.post_request::<ConnectRequest, ConnectResponse>(
            format!("http://{server_address}/api/v1/connect"),
            request
        ).await?;

        // Validate response
        if !response.validate(proof_seed)? {
            return Err(Error::InvalidProofSeedSignature);
        }

        // Check response status
        match response.0 {
            Response::Success { .. } => {
                let client = ConnectedClient {
                    http_client: self.http_client.clone(),
                    driver: self.driver.clone(),
                    connected_server: ServerApiRecord {
                        public_key: server_public,
                        address: server_address.to_string()
                    },
                    connection_certificate: certificate
                };

                Ok(client)
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

#[derive(Debug, Clone, Hash)]
/// Connected client HTTP middleware
/// 
/// This struct is used to perform HTTP REST API requests
/// to the servers from the name of inner client driver.
/// 
/// This struct is derived from the `Client` middlewire
/// and provides advanced methods that depend on connection
/// certificate.
pub struct ConnectedClient<T> {
    http_client: Arc<T>,
    driver: Arc<ClientDriver>,
    connected_server: ServerApiRecord,
    connection_certificate: ConnectionCertificate
}

impl<T: HttpClient> ConnectedClient<T> {
    #[inline]
    pub fn http_client(&self) -> Arc<T> {
        self.http_client.clone()
    }

    #[inline]
    pub fn http_client_ref(&self) -> &T {
        &self.http_client
    }

    #[inline]
    pub fn driver(&self) -> Arc<ClientDriver> {
        self.driver.clone()
    }

    #[inline]
    pub fn driver_ref(&self) -> &ClientDriver {
        &self.driver
    }

    #[inline]
    pub fn connected_server(&self) -> &ServerApiRecord {
        &self.connected_server
    }

    #[inline]
    pub fn connection_certificate(&self) -> &ConnectionCertificate {
        &self.connection_certificate
    }

    /// Construct new `Client` struct from the protocol's paper.
    /// 
    /// Service function used by other methods in this struct.
    pub fn get_client(&self) -> ClientApiRecord {
        ClientApiRecord::new(
            self.driver.secret_key().public_key(),
            self.connection_certificate.clone(),
            self.driver.info().clone()
        )
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(ret, skip_all, fields(
        server
    )))]
    /// Announce remote server server about yourself.
    /// 
    /// This method will perform `POST /api/v1/announce` request.
    /// 
    /// - `server` should contain address of the server
    ///   you want to announce about the current client.
    pub async fn announce(&self, server: impl std::fmt::Display) -> Result<(), Error> {
        #[cfg(feature = "tracing")]
        tracing::debug!("Sending POST /api/v1/announce request");

        // Prepare announce request
        let request = AnnounceRequest::client(
            self.driver.secret_key(),
            self.get_client(),
            self.connected_server.clone()
        );

        let proof_seed = request.0.proof_seed;

        // Send request
        let response = self.http_client.post_request::<AnnounceRequest, AnnounceResponse>(
            format!("http://{server}/api/v1/announce"),
            request
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
        client_public = client_public.to_base64(),
        client_type = ?client_type
    )))]
    /// Lookup given client.
    /// 
    /// This method will perform `POST /api/v1/lookup` request.
    /// 
    /// - `client_public` must be a public key of the client
    ///   you need to find.
    /// 
    /// - `client_type` is an optional filter of the type
    ///   of the client you need to find.
    /// 
    /// This method will return a tuple of client and server
    /// info and whether the client is available. Availability
    /// of the client is decided by the server from which you
    /// have obtained this info. You should not use this info
    /// if you don't trust this server.
    /// 
    /// This method will keep requesting servers until no more
    /// hints returned or needed client is found.
    pub async fn lookup(&self, client_public: PublicKey, client_type: Option<ClientType>) -> Result<Option<(ClientApiRecord, ServerApiRecord, bool)>, Error> {
        // Prepare lookup request
        let request = LookupRequest::new(self.driver.secret_key(), client_public, client_type);

        let proof_seed = request.0.proof_seed;

        // Queue of search hints
        let mut queue = VecDeque::from([
            self.connected_server.address.clone()
        ]);

        // Store used servers to prevent infinite lookup loops
        let mut used_servers = HashSet::new();

        while let Some(server_address) = queue.pop_front() {
            // Skip server if it was already used
            if used_servers.contains(&server_address) {
                continue;
            }

            #[cfg(feature = "tracing")]
            tracing::debug!(server_address, "Sending POST /api/v1/lookup request");

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
        receiver = receiver.to_base64(),
        channel = channel.to_string(),
        message = format!("{}: {}", message.encoding, message.content)
    )))]
    /// Send a message to remote client.
    /// 
    /// This method will perform `POST /api/v1/send` request.
    /// 
    /// - `receiver` must be a public key of the message receiver.
    /// 
    /// - `channel` must be a string name of the channel to which
    ///   we want to send this message. Channels allow to differ
    ///   messages streams for different applications and their
    ///   parts (modules).
    /// 
    /// - `message` should contain the message you want to send.
    pub async fn send(&self, receiver: PublicKey, channel: impl ToString, message: Message) -> Result<(), Error> {
        #[cfg(feature = "tracing")]
        tracing::debug!("Sending POST /api/v1/send request");

        // Prepare send message request
        let client = ClientApiRecord::new(
            self.driver.secret_key().public_key(),
            self.connection_certificate.clone(),
            ClientInfo::thin()
        );

        let sender = Sender::new(client, self.connected_server.clone());

        let request = SendRequest::new(
            self.driver.secret_key(),
            sender,
            receiver,
            channel,
            message
        );

        let proof_seed = request.0.proof_seed;

        // Send request
        let response = self.http_client.post_request::<SendRequest, SendResponse>(
            format!("http://{}/api/v1/send", &self.connected_server.address),
            request
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
        channel = channel.to_string(),
        limit
    )))]
    /// Poll (read and delete) messages from the server's inbox.
    /// 
    /// This method will perform `POST /api/v1/poll` request.
    /// 
    /// - `channel` must contain string name of the messages channel.
    /// 
    /// - `limit` should contain amount of messages you want to poll,
    ///   or `None` if all the stored. This param is used by the
    ///   remote server which may not respect it, so you should
    ///   consider implementing few manual checks if you don't trust
    ///   the server you connected to.
    /// 
    /// This method will return vector of polled messages and
    /// amount of remaining messages in the server's inbox.
    pub async fn poll(&self, channel: impl ToString, limit: Option<u64>) -> Result<(Vec<MessageInfo>, u64), Error> {
        #[cfg(feature = "tracing")]
        tracing::debug!("Sending POST /api/v1/poll request");

        // Prepare poll request
        let request = PollRequest::new(self.driver.secret_key(), channel, limit);

        let proof_seed = request.0.proof_seed;

        // Send request
        let response = self.http_client.post_request::<PollRequest, PollResponse>(
            format!("http://{}/api/v1/poll", &self.connected_server.address),
            request
        ).await?;

        // Validate response
        if !response.validate(proof_seed)? {
            return Err(Error::InvalidProofSeedSignature);
        }

        // Check response status
        match response.0 {
            Response::Success { response, .. } => {
                Ok((response.messages, response.remaining))
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
