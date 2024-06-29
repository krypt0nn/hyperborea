use crate::crypto::PublicKey;

use crate::rest_api::clients::Client;
use crate::rest_api::servers::Server;
use crate::rest_api::connect::ClientType;

#[async_trait::async_trait]
/// Router is a struct that implements network clients
/// and servers indexing, listing and lookup operations.
pub trait MessagesQueue {
    // async fn index_message(&self, from_client: Client, from_server: Server, to_client: Client, message: );
}
