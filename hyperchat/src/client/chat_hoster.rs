use std::str::FromStr;
use std::sync::Arc;
use std::sync::RwLock;
use std::collections::{HashMap, VecDeque};

use hyperelm::prelude::*;
use hyperborealib::prelude::*;

use crate::params::Params;

use super::*;

pub struct ChatHosterState {
    pub members: RwLock<HashMap<PublicKey, String>>,
    pub history: RwLock<VecDeque<ChatHistoryBlock>>
}

pub struct ChatHosterApp {
    params: ClientAppParams,
    middlewire: ClientMiddleware<ReqwestHttpClient>,
    state: Arc<ChatHosterState>
}

impl ChatHosterApp {
    pub fn from_params(params: &Params) -> anyhow::Result<Self> {
        let client_secret = SecretKey::from_base64(&params.client_secret)?;
        let server_public = PublicKey::from_base64(&params.client_server_public)?;

        let encoding = MessageEncoding::from_str(&params.room_encoding)
            .map_err(|err| anyhow::anyhow!("Failed to find chat room endoding: {err}"))?;

        Ok(Self {
            params: ClientAppParams::builder()
                .channel("hyperchat")
                .encoding(encoding)
                .delay(std::time::Duration::from_millis(params.room_sync_delay))
                .client(client_secret.clone())
                .server(server_public, &params.client_server_address)
                .build()
                .unwrap(),

            middlewire: ClientMiddleware::new(
                ReqwestHttpClient::default(),
                ClientDriver::thin(client_secret)
            ),

            state: Arc::new(ChatHosterState {
                members: RwLock::new(HashMap::new()),
                history: RwLock::new(VecDeque::new())
            })
        })
    }
}

impl ClientApp for ChatHosterApp {
    build_client!(
        input: ChatMemberRequest => ChatHosterResponse, ChatMemberMessage;
        output: () => (), ();

        client: ReqwestHttpClient;
        state: ChatHosterState;
        error: anyhow::Error;

        requests: {
            ChatMemberRequest::Join { username } => |state: Arc<Self::State>, info: MessageInfo| async move {
                log::info!("[app][Join] New member joined chat: {username}");
                log::info!("[app][Join]   Public key : {}", info.sender.client.public_key.to_base64());

                state.members.write()
                    .expect("Failed to update application state (members list)")
                    .insert(info.sender.client.public_key.clone(), username.clone());

                let mut history = state.history.write()
                    .expect("Failed to update application state (history)");

                let last_id = match history.len().checked_sub(1) {
                    Some(i) => history[i].id,
                    None => 0
                };

                history.push_back(ChatHistoryBlock {
                    id: last_id + 1,
                    timestamp: timestamp(),
                    body: ChatHistoryBlockBody::MemberJoin {
                        public_key: info.sender.client.public_key.clone(),
                        username
                    }
                });

                Ok(ChatHosterResponse::JoinResponse {
                    members: state.members.read()
                        .expect("Failed to read application state (members list)")
                        .clone(),

                    history: history.clone()
                })
            }

            ChatMemberRequest::GetMembers => |state: Arc<Self::State>, info: MessageInfo| async move {
                log::info!("[app][GetMembers] Get chat members list");
                log::info!("[app][GetMembers]   Public key : {}", info.sender.client.public_key.to_base64());

                Ok(ChatHosterResponse::Members {
                    members: state.members.read()
                        .expect("Failed to read application state (members list)")
                        .clone()
                })
            }

            ChatMemberRequest::GetHistory { since_id } => |state: Arc<Self::State>, info: MessageInfo| async move {
                log::info!("[app][GetMembers] Get chat history");
                log::info!("[app][GetMembers]   Public key : {}", info.sender.client.public_key.to_base64());

                let mut history = state.history.read()
                    .expect("Failed to read application state (history)")
                    .iter()
                    .filter(|record| record.id >= since_id)
                    .cloned()
                    .collect::<Vec<_>>();

                history.sort_by(|a, b| a.id.cmp(&b.id));

                Ok(ChatHosterResponse::History {
                    history: VecDeque::from(history)
                })
            }
        };

        messages: {
            ChatMemberMessage::SendMessage { message } => |state: Arc<Self::State>, info: MessageInfo| async move {
                log::info!("[app][SendMessage] New chat message:");
                log::info!("[app][SendMessage]   Public key : {}", info.sender.client.public_key.to_base64());
                log::info!("[app][SendMessage]   Message    : {message}");

                let mut history = state.history.write()
                    .expect("Failed to update application state (history)");

                let last_id = match history.len().checked_sub(1) {
                    Some(i) => history[i].id,
                    None => 0
                };

                history.push_back(ChatHistoryBlock {
                    id: last_id + 1,
                    timestamp: timestamp(),
                    body: ChatHistoryBlockBody::MemberSendMessage {
                        public_key: info.sender.client.public_key.clone(),
                        message
                    }
                });

                Ok(())
            }

            ChatMemberMessage::Leave => |state: Arc<Self::State>, info: MessageInfo| async move {
                log::info!("[app][Leave] Member left chat");
                log::info!("[app][Leave]   Public key : {}", info.sender.client.public_key.to_base64());

                state.members.write()
                    .expect("Failed to update application state (members list)")
                    .remove(&info.sender.client.public_key);

                let mut history = state.history.write()
                    .expect("Failed to update application state (history)");

                let last_id = match history.len().checked_sub(1) {
                    Some(i) => history[i].id,
                    None => 0
                };

                history.push_back(ChatHistoryBlock {
                    id: last_id + 1,
                    timestamp: timestamp(),
                    body: ChatHistoryBlockBody::MemberLeave {
                        public_key: info.sender.client.public_key.clone()
                    }
                });

                Ok(())
            }
        };
    );

    #[inline]
    fn get_params(&self) ->  &ClientAppParams {
        &self.params
    }

    #[inline]
    fn get_middlewire(&self) ->  &ClientMiddleware<Self::HttpClient> {
        &self.middlewire
    }

    #[inline]
    fn get_state(&self) -> Arc<Self::State> {
        self.state.clone()
    }
}
