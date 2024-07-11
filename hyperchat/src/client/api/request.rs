use hyperelm::prelude::*;

use hyperborealib::impl_as_json;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
/// Request sent by the chat members to the chat hoster.
/// 
/// ```text
///  ┌──────────┐  Request  ┌──────────┐ 
///  │          ├──────────►│          │ 
///  │  Member  │           │  Hoster  │ 
///  │          │◄──────────┤          │ 
///  └──────────┘           └──────────┘ 
/// ```
pub enum ChatMemberRequest {
    Join {
        username: String
    },

    GetMembers,

    GetHistory {
        since_id: u64
    }
}

impl_as_json!(ChatMemberRequest);
