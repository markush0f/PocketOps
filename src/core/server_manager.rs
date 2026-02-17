use crate::models::ManagedServer;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};

const SERVERS_FILE: &str = "servers.json";

#[derive(Clone)]
pub struct ServerManager {
    servers: Arc<Mutex<HashMap<String, ManagedServer>>>,
    file_path: String,
}

impl ServerManager {
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
            servers.insert("local".to_string(), ManagedServer {
                id: "local-auto".to_string(),
                hostname: "127.0.0.1".to_string(),
                ip_address: "127.0.0.1".to_string(),
                port: 22,
                ssh_user: user,
                password: None,
            });
            // We don't save automatically to avoid cluttering if not desired, 
            // but it's available in memory for this session (or until restart if not saved).
            // To make it persistent/robust, let's just leave it in memory or save. 
            // Let's NOT save it generally, so it dynamically adapts if the user changes (e.g. running on different machine).
        }
        drop(servers);

        manager
    }

    fn load(&mut self) {
        if Path::new(&self.file_path).exists() {
            if let Ok(content) = fs::read_to_string(&self.file_path) {
                if let Ok(servers) = serde_json::from_str::<HashMap<String, ManagedServer>>(&content) {
                    *self.servers.lock().unwrap() = servers;
                }
            }
        }
    }

    fn save(&self) {
        let servers = self.servers.lock().unwrap();
        if let Ok(content) = serde_json::to_string_pretty(&*servers) {
            let _ = fs::write(&self.file_path, content);
        }
    }

    pub fn add_server(&self, alias: String, host: String, user: String, port: u16, password: Option<String>) {
        let server = ManagedServer {
            id: uuid::Uuid::new_v4().to_string(),
            hostname: host,
            ip_address: String::new(), // Will be resolved or same as hostname
            port,
            ssh_user: user,
            password,
        };
        
        // For simplicity, using hostname as IP address usually works for SSH
        // Or we could do a DNS lookup here. Let's just store hostname in ip_address for now if it's an IP.
        // Actually, let's keep it simple: hostname is the address we connect to.
        let mut server = server;
        server.ip_address = server.hostname.clone(); 

        self.servers.lock().unwrap().insert(alias, server);
        self.save();
    }

    pub fn remove_server(&self, alias: &str) -> bool {
        let mut servers = self.servers.lock().unwrap();
        let result = servers.remove(alias).is_some();
        drop(servers); // Unlock before save
        if result {
            self.save();
        }
        result
    }

    pub fn get_server(&self, alias: &str) -> Option<ManagedServer> {
        self.servers.lock().unwrap().get(alias).cloned()
    }

    pub fn list_servers(&self) -> Vec<(String, ManagedServer)> {
        self.servers.lock().unwrap()
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }
}
