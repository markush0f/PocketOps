use crate::models::ManagedServer;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};

const SERVERS_FILE: &str = "servers.json";

/// Manages the collection of servers that PocketSentinel can interact with.
///
/// This struct handles loading/saving servers to JSON, adding, removing,
/// and retrieving server details. It uses an `Arc<Mutex<...>>` to allow safe
/// concurrent access if needed in the future.
#[derive(Clone)]
pub struct ServerManager {
    servers: Arc<Mutex<HashMap<String, ManagedServer>>>,
    file_path: String,
}

impl ServerManager {
    /// Creates a new `ServerManager` and loads existing servers from disk.
    ///
    /// If no servers are configured, it automatically adds a 'local' server
    /// configuration for the current machine to facilitate testing and usage.
    pub fn new() -> Self {
        let mut manager = ServerManager {
            servers: Arc::new(Mutex::new(HashMap::new())),
            file_path: SERVERS_FILE.to_string(),
        };
        manager.load();

        // Auto-configure 'local' server if missing
        let mut servers = manager.servers.lock().unwrap();
        if !servers.contains_key("local") {
            let user = std::env::var("USER").unwrap_or_else(|_| "root".to_string());
            servers.insert(
                "local".to_string(),
                ManagedServer {
                    id: "local-auto".to_string(),
                    hostname: "127.0.0.1".to_string(),
                    ip_address: "127.0.0.1".to_string(),
                    port: 22,
                    ssh_user: user,
                    password: None,
                },
            );
        }
        drop(servers);

        manager
    }

    /// Loads the server configurations from the JSON file.
    fn load(&mut self) {
        if Path::new(&self.file_path).exists() {
            if let Ok(content) = fs::read_to_string(&self.file_path) {
                if let Ok(servers) =
                    serde_json::from_str::<HashMap<String, ManagedServer>>(&content)
                {
                    *self.servers.lock().unwrap() = servers;
                }
            }
        }
    }

    /// Saves the current server configurations to the JSON file.
    fn save(&self) {
        let servers = self.servers.lock().unwrap();
        if let Ok(content) = serde_json::to_string_pretty(&*servers) {
            let _ = fs::write(&self.file_path, content);
        }
    }

    /// Adds a new server to the manager and saves it.
    ///
    /// # Arguments
    ///
    /// * `alias` - A friendly name for the server (e.g., "prod-db").
    /// * `host` - The hostname or IP address.
    /// * `user` - The SSH username.
    /// * `port` - The SSH port (usually 22).
    /// * `password` - Optional password for authentication (keys are preferred).
    pub fn add_server(
        &self,
        alias: String,
        host: String,
        user: String,
        port: u16,
        password: Option<String>,
    ) {
        let server = ManagedServer {
            id: uuid::Uuid::new_v4().to_string(),
            hostname: host,
            ip_address: String::new(), // Will be resolved or same as hostname
            port,
            ssh_user: user,
            password,
        };

        let mut server = server;
        server.ip_address = server.hostname.clone();

        self.servers.lock().unwrap().insert(alias, server);
        self.save();
    }

    /// Removes a server by its alias.
    ///
    /// Returns `true` if the server was found and removed, `false` otherwise.
    pub fn remove_server(&self, alias: &str) -> bool {
        let mut servers = self.servers.lock().unwrap();
        let result = servers.remove(alias).is_some();
        drop(servers); // Unlock before save
        if result {
            self.save();
        }
        result
    }

    /// Retrieves a server configuration by its alias.
    pub fn get_server(&self, alias: &str) -> Option<ManagedServer> {
        self.servers.lock().unwrap().get(alias).cloned()
    }

    /// Lists all configured servers.
    ///
    /// Returns a vector of tuples containing the alias and the `ManagedServer` struct.
    pub fn list_servers(&self) -> Vec<(String, ManagedServer)> {
        self.servers
            .lock()
            .unwrap()
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }
}
