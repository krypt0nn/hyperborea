use hyperelm::prelude::*;

use hyperborealib::crypto::*;

mod api;
mod chat_member;
mod chat_hoster;

pub use api::*;
pub use chat_member::*;
pub use chat_hoster::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct ChatHistoryBlock {
    pub id: u64,
    pub timestamp: u64,
    pub body: ChatHistoryBlockBody
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum ChatHistoryBlockBody {
    MemberJoin {
        public_key: PublicKey,
        username: String
    },

    MemberSendMessage {
        public_key: PublicKey,
        message: String
    },

    MemberLeave {
        public_key: PublicKey
    }
}
