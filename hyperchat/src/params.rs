use serde_json::{json, Value as Json};

use hyperborealib::crypto::{
    SecretKey,
    safe_random_u64
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Params {
    pub client_secret: String,

    pub server_secret: String,
    pub server_local_address: String,
    pub server_exposed_address: String,

    pub bootstrap_addresses: Vec<String>,
    pub bootstrap_traversal_delay: u64,

    pub room_secret_key: String,
    pub room_name: String,
    pub room_username: String,
    pub room_lookup_delay: u64,
    pub room_sync_delay: u64,
    pub room_encoding: String
}

impl Default for Params {
    fn default() -> Self {
        Self {
            client_secret: SecretKey::random().to_base64(),

            server_secret: SecretKey::random().to_base64(),
            server_local_address: String::from("127.0.0.1:51234"),
            server_exposed_address: String::from("<you have to edit this>"),

            bootstrap_addresses: Vec::new(),
            bootstrap_traversal_delay: 60 * 5,

            room_secret_key: SecretKey::random().to_base64(),
            room_name: format!("Room #{}", safe_random_u64() % 9000 + 1000),
            room_username: format!("User #{}", safe_random_u64() % 9000 + 1000),
            room_lookup_delay: 15,
            room_sync_delay: 1000,
            room_encoding: String::from("base64/chacha20-poly1305")
        }
    }
}

pub async fn read() -> anyhow::Result<Params> {
    if !std::path::PathBuf::from("params.json").exists() {
        write(&Params::default()).await?;
    }

    let params = tokio::fs::read("params.json").await?;
    let params = serde_json::from_slice::<Json>(&params)?;

    Ok(Params {
        client_secret: params["client"]["secret_key"].as_str().unwrap().to_string(),

        server_secret: params["server"]["secret_key"].as_str().unwrap().to_string(),
        server_local_address: params["server"]["local_address"].as_str().unwrap().to_string(),
        server_exposed_address: params["server"]["exposed_address"].as_str().unwrap().to_string(),

        bootstrap_addresses: params["bootstrap"]["addresses"].as_array().unwrap()
            .iter()
            .flat_map(Json::as_str)
            .map(String::from)
            .collect(),

        bootstrap_traversal_delay: params["bootstrap"]["traversal_delay"].as_u64().unwrap(),

        room_secret_key: params["room"]["secret_key"].as_str().unwrap().to_string(),
        room_name: params["room"]["name"].as_str().unwrap().to_string(),
        room_username: params["room"]["username"].as_str().unwrap().to_string(),
        room_lookup_delay: params["room"]["lookup_delay"].as_u64().unwrap(),
        room_sync_delay: params["room"]["sync_delay"].as_u64().unwrap(),
        room_encoding: params["room"]["encoding"].as_str().unwrap().to_string()
    })
}

pub async fn write(params: &Params) -> anyhow::Result<()> {
    let params = serde_json::to_string_pretty(&json!({
        "client": {
            "secret_key": params.client_secret
        },
        "server": {
            "secret_key": params.server_secret,
            "local_address": params.server_local_address,
            "exposed_address": params.server_exposed_address
        },
        "bootstrap": {
            "addresses": params.bootstrap_addresses,
            "traversal_delay": params.bootstrap_traversal_delay
        },
        "room": {
            "secret_key": params.room_secret_key,
            "name": params.room_name,
            "username": params.room_username,
            "lookup_delay": params.room_lookup_delay,
            "sync_delay": params.room_sync_delay,
            "encoding": params.room_encoding
        }
    }))?;

    tokio::fs::write("params.json", params).await?;

    Ok(())
}
