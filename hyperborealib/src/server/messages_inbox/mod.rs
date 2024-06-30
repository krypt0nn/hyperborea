use crate::crypto::PublicKey;

use crate::rest_api::prelude::*;

pub mod basic_inbox;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InboxRecord {
    pub sender: Sender,
    pub channel: String,
    pub message: Message,
    pub received_at: u64
}

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
    async fn poll_messages(&self, receiver: PublicKey, channel: String, limit: Option<usize>) -> (Vec<InboxRecord>, usize);
}
