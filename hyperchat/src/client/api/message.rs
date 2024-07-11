use hyperelm::prelude::*;

use hyperborealib::impl_as_json;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
/// Message sent by the chat members to the chat hoster.
/// 
/// ```text
///  ┌──────────┐           ┌──────────┐ 
///  │          │  Message  │          │ 
///  │  Member  ├──────────►│  Hoster  │ 
///  │          │           │          │ 
///  └──────────┘           └──────────┘ 
/// ```
pub enum ChatMemberMessage {
    SendMessage {
        message: String
    },

    Leave
}

impl_as_json!(ChatMemberMessage);
