use std::str::FromStr;
use std::sync::Arc;
use std::sync::RwLock;
use std::collections::{HashMap, VecDeque};

use hyperelm::prelude::*;
use hyperborealib::prelude::*;

use crate::params::Params;

use super::*;

pub struct ChatMemberState {
    pub members: RwLock<HashMap<PublicKey, String>>,
    pub history: RwLock<VecDeque<ChatHistoryBlock>>
}

pub struct ChatMemberApp {
    params: ClientAppParams,
    middleware: ClientMiddleware<ReqwestHttpClient>,
    state: Arc<ChatMemberState>
}

impl ChatMemberApp {
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

            middleware: ClientMiddleware::new(
                ReqwestHttpClient::default(),
                ClientDriver::thin(client_secret)
            ),

            state: Arc::new(ChatMemberState {
                members: RwLock::new(HashMap::new()),
                history: RwLock::new(VecDeque::new())
            })
        })
    }
}

impl ClientApp for ChatMemberApp {
    build_client!(
        input: () => (), ();
        output: ChatMemberRequest => ChatHosterResponse, ChatMemberMessage;

        client: ReqwestHttpClient;
        state: ChatMemberState;
        error: anyhow::Error;

        requests: {};
        messages: {};
    );

    #[inline]
    fn get_params(&self) ->  &ClientAppParams {
        &self.params
    }

    #[inline]
    fn get_middleware(&self) ->  &ClientMiddleware<Self::HttpClient> {
        &self.middleware
    }

    #[inline]
    fn get_state(&self) -> Arc<Self::State> {
        self.state.clone()
    }
}
