use crate::crypto::asymmetric::PublicKey;

use crate::rest_api::prelude::*;

pub mod basic_inbox;

#[async_trait::async_trait]
/// MessagesQueue is a struct that stores messages
/// sent by external clients and meant to be read
/// by local clients.
pub trait MessagesInbox {
    /// Add new message to the inbox.
    async fn add_message(&self, sender: Sender, receiver: PublicKey, channel: String, message: Message);

    /// Read client's inbox, applying given filters.
    /// 
    /// Return list of read messages and number of remained.
    /// 
    /// This method will remove read messages from the inbox.
    async fn poll_messages(&self, receiver: PublicKey, channel: String, limit: Option<u64>) -> (Vec<MessageInfo>, u64);
}
