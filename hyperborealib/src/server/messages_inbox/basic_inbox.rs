use std::time::Duration;

use moka::future::Cache;

use crate::time::timestamp;

use crate::crypto::PublicKey;
use crate::rest_api::prelude::*;

use super::{MessagesInbox, InboxRecord};

#[derive(Debug, Clone)]
pub struct BasicInbox {
    pub inbox: Cache<PublicKey, Vec<InboxRecord>>
}

impl Default for BasicInbox {
    #[inline]
    fn default() -> Self {
        Self::new(Duration::from_secs(60 * 60 * 24))
    }
}

impl BasicInbox {
    pub fn new(ttl: Duration) -> Self {
        #[cfg(feature = "tracing")]
        tracing::trace!("Building new BasicInbox with {} seconds lifetime", ttl.as_secs());

        Self {
            inbox: Cache::builder()
                .time_to_idle(ttl)
                .build()
        }
    }
}

#[async_trait::async_trait]
impl MessagesInbox for BasicInbox {
    async fn add_message(&self, sender: Sender, receiver: PublicKey, channel: String, message: Message) {
        let mut inbox = self.inbox.get(&receiver).await
            .unwrap_or_default();

        inbox.push(InboxRecord {
            sender,
            channel,
            message,
            received_at: timestamp()
        });

        self.inbox.insert(receiver, inbox).await;
    }

    async fn poll_messages(&self, receiver: PublicKey, channel: String, limit: Option<usize>) -> (Vec<InboxRecord>, usize) {
        let mut inbox = self.inbox.get(&receiver).await
            .unwrap_or_default();

        let mut messages = Vec::new();

        let mut limit = limit.unwrap_or(inbox.len());
        let mut i = 0;

        while inbox.len() > i && limit > 0 {
            let record = &inbox[i];

            if record.channel == channel {
                limit -= 1;

                messages.push(inbox.remove(i));
            }

            else {
                i += 1;
            }
        }

        let ramined = inbox.len();

        self.inbox.insert(receiver, inbox).await;

        (messages, ramined)
    }
}
