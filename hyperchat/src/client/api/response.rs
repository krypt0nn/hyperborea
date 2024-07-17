use std::collections::{HashMap, VecDeque};

use hyperelm::prelude::*;
use hyperborealib::prelude::*;

use hyperborealib::impl_as_json;

use crate::client::ChatHistoryBlock;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
/// Response sent by the chat hoster to the chat members.
/// 
/// ```text
///  ┌──────────┐           ┌──────────┐ 
///  │          ├──────────►│          │ 
///  │  Member  │           │  Hoster  │ 
///  │          │◄──────────┤          │ 
///  └──────────┘ Response  └──────────┘ 
/// ```
pub enum ChatHosterResponse {
    JoinResponse {
        members: HashMap<PublicKey, String>,
        history: VecDeque<ChatHistoryBlock>
    },

    Members {
        members: HashMap<PublicKey, String>
    },

    History {
        history: VecDeque<ChatHistoryBlock>
    }
}

impl_as_json!(ChatHosterResponse);
