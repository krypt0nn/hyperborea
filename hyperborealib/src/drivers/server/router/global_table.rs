use std::path::PathBuf;

use serde_json::{json, Value as Json};

use crate::rest_api::prelude::*;
use crate::time::timestamp;

use super::Router;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Json(#[from] AsJsonError),

    #[error(transparent)]
    Serialize(#[from] serde_json::Error)
}

#[derive(Debug, Clone)]
/// Global Table Router stores all the record in a separate
/// files within the given folder.
pub struct GlobalTableRouter {
    /// Path to the routing table's folder.
    pub storage_folder: PathBuf
}

impl GlobalTableRouter {
    pub async fn new(storage_folder: impl Into<PathBuf>) -> std::io::Result<Self> {
        let storage_folder = storage_folder.into();

        #[cfg(feature = "tracing")]
        tracing::trace!("Building new GlobalTableRouter in {:?}", storage_folder);

        tokio::fs::create_dir_all(storage_folder.join("local")).await?;
        tokio::fs::create_dir_all(storage_folder.join("remote")).await?;
        tokio::fs::create_dir_all(storage_folder.join("servers")).await?;

        Ok(Self {
            storage_folder
        })
    }
}

#[async_trait::async_trait]
impl Router for GlobalTableRouter {
    type Error = Error;

    async fn index_local_client(&self, client: Client) -> Result<bool, Self::Error> {
        let path = self.storage_folder
            .join("local")
            .join(client.public_key.to_base64());

        let client = json!({
            "indexed_at": timestamp(),
            "client": client.to_json()?
        });

        tokio::fs::write(path, serde_json::to_vec(&client)?).await?;

        Ok(true)
    }

    async fn index_remote_client(&self, client: Client, server: Server) -> Result<bool, Self::Error> {
        let path = self.storage_folder
            .join("remote")
            .join(client.public_key.to_base64());

        let record = json!({
            "indexed_at": timestamp(),
            "client": client,
            "server": server
        });

        tokio::fs::write(path, serde_json::to_vec(&record)?).await?;

        Ok(true)
    }

    async fn index_server(&self, server: Server) -> Result<bool, Self::Error> {
        let path = self.storage_folder
            .join("servers")
            .join(server.public_key.to_base64());

        let server = json!({
            "indexed_at": timestamp(),
            "server": server.to_json()?
        });

        tokio::fs::write(path, serde_json::to_vec(&server)?).await?;

        Ok(true)
    }

    async fn local_clients(&self) -> Result<Vec<Client>, Self::Error> {
        let mut clients = Vec::new();

        let folder = self.storage_folder
            .join("local");

        let mut entries = tokio::fs::read_dir(folder).await?;

        while let Some(entry) = entries.next_entry().await? {
            let entry = tokio::fs::read(entry.path()).await?;
            let record = serde_json::from_slice::<Json>(&entry)?;

            let client = Client::from_json(&record["client"])?;

            clients.push(client);
        }

        Ok(clients)
    }

    async fn remote_clients(&self) -> Result<Vec<(Client, Server)>, Self::Error> {
        let mut clients = Vec::new();

        let folder = self.storage_folder
            .join("remote");

        let mut entries = tokio::fs::read_dir(folder).await?;

        while let Some(entry) = entries.next_entry().await? {
            let entry = tokio::fs::read(entry.path()).await?;
            let record = serde_json::from_slice::<Json>(&entry)?;

            let client = Client::from_json(&record["client"])?;
            let server = Server::from_json(&record["server"])?;

            clients.push((client, server));
        }

        Ok(clients)
    }

    async fn servers(&self) -> Result<Vec<Server>, Self::Error> {
        let mut servers = Vec::new();

        let folder = self.storage_folder
            .join("servers");

        let mut entries = tokio::fs::read_dir(folder).await?;

        while let Some(entry) = entries.next_entry().await? {
            let entry = tokio::fs::read(entry.path()).await?;
            let record = serde_json::from_slice::<Json>(&entry)?;

            let server = Server::from_json(&record["server"])?;

            servers.push(server);
        }

        Ok(servers)
    }
}

#[cfg(test)]
mod tests {
    use crate::rest_api::types::client::tests::get_client;
    use crate::rest_api::types::server::tests::get_server;

    use super::*;

    #[tokio::test]
    async fn index_lookup() -> std::io::Result<()> {
        let temp = std::env::temp_dir()
            .join("global-table-router-test");

        if temp.exists() {
            std::fs::remove_dir_all(&temp)?;
        }

        std::fs::create_dir(&temp)?;

        let table = GlobalTableRouter::new(&temp).await?;

        let local = vec![get_client(); 32];
        let remote = vec![(get_client(), get_server()); 32];
        let servers = vec![get_server(); 32];

        // Index clients

        for client in &local {
            table.index_local_client(
                client.to_owned()
            ).await.unwrap();
        }

        for (client, server) in &remote {
            table.index_remote_client(
                client.to_owned(),
                server.to_owned()
            ).await.unwrap();
        }

        for server in &servers {
            table.index_server(
                server.to_owned()
            ).await.unwrap();
        }

        // Lookup clients

        for client in local {
            let found = table.lookup_local_client(
                &client.public_key,
                Some(client.info.client_type.clone())
            ).await.unwrap().unwrap();

            assert_eq!(client, found.0);
        }

        for (client, server) in remote {
            let found = table.lookup_remote_client(
                &client.public_key,
                Some(client.info.client_type.clone())
            ).await.unwrap().unwrap();

            assert_eq!(client, found.0);
            assert_eq!(server, found.1);
        }

        for server in servers {
            let found = table.lookup_server(
                &server.public_key
            ).await.unwrap().unwrap();

            assert_eq!(server, found.0);
        }

        Ok(())
    }
}
