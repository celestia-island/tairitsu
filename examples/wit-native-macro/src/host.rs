//! Approach A Host: Demonstrating macro-generated WIT interfaces
//!
//! This shows how the wit_interface! macro generates enums with zero boilerplate

use log::info;
use tairitsu::{wit_interface, wit_registry::WitCommandHandler};

// Generate three separate interfaces using the macro
wit_interface! {
    interface filesystem {
        read: func(path: String) -> Result<Vec<u8>, String>;
        write: func(path: String, data: Vec<u8>) -> Result<(), String>;
        delete: func(path: String) -> Result<(), String>;
        list: func(directory: String) -> Result<Vec<String>, String>;
    }
}

wit_interface! {
    interface network {
        http_get: func(url: String) -> Result<String, String>;
        http_post: func(url: String, body: String) -> Result<String, String>;
    }
}

wit_interface! {
    interface database {
        query: func(sql: String) -> Result<String, String>;
        execute: func(sql: String) -> Result<u64, String>;
    }
}

// Filesystem handler with persistent storage
struct FilesystemHandler {
    storage: std::collections::HashMap<String, Vec<u8>>,
}

impl FilesystemHandler {
    fn new() -> Self {
        Self {
            storage: std::collections::HashMap::new(),
        }
    }
}

impl WitCommandHandler<FilesystemCommands> for FilesystemHandler {
    fn execute(&mut self, command: &FilesystemCommands) -> Result<FilesystemResponse, String> {
        match command {
            FilesystemCommands::Read { path } => {
                let data = self
                    .storage
                    .get(path)
                    .cloned()
                    .ok_or_else(|| format!("File not found: {}", path))?;
                Ok(FilesystemResponse::Read(Ok(data)))
            }
            FilesystemCommands::Write { path, data } => {
                self.storage.insert(path.clone(), data.clone());
                info!("[FS] Wrote {} bytes to {}", data.len(), path);
                Ok(FilesystemResponse::Write(Ok(())))
            }
            FilesystemCommands::Delete { path } => {
                self.storage
                    .remove(path)
                    .ok_or_else(|| format!("File not found: {}", path))?;
                info!("[FS] Deleted {}", path);
                Ok(FilesystemResponse::Delete(Ok(())))
            }
            FilesystemCommands::List { directory } => {
                let files: Vec<String> = self
                    .storage
                    .keys()
                    .filter(|k| k.starts_with(directory))
                    .cloned()
                    .collect();
                info!("[FS] Listed {} files in {}", files.len(), directory);
                Ok(FilesystemResponse::List(Ok(files)))
            }
        }
    }
}

// Network handler
struct NetworkHandler;

impl WitCommandHandler<NetworkCommands> for NetworkHandler {
    fn execute(&mut self, command: &NetworkCommands) -> Result<NetworkResponse, String> {
        match command {
            NetworkCommands::Http_get { url } => {
                info!("[NET] HTTP GET {}", url);
                Ok(NetworkResponse::Http_get(Ok(format!(
                    "{{\"status\":\"ok\",\"url\":\"{}\"}}",
                    url
                ))))
            }
            NetworkCommands::Http_post { url, body } => {
                info!("[NET] HTTP POST {} ({} bytes)", url, body.len());
                Ok(NetworkResponse::Http_post(Ok(format!(
                    "{{\"status\":\"ok\",\"posted\":{}}}",
                    body.len()
                ))))
            }
        }
    }
}

// Database handler
struct DatabaseHandler;

impl WitCommandHandler<DatabaseCommands> for DatabaseHandler {
    fn execute(&mut self, command: &DatabaseCommands) -> Result<DatabaseResponse, String> {
        match command {
            DatabaseCommands::Query { sql } => {
                info!("[DB] Query: {}", sql);
                Ok(DatabaseResponse::Query(Ok(
                    "[[\"id\",1],[\"name\",\"test\"]]".to_string(),
                )))
            }
            DatabaseCommands::Execute { sql } => {
                info!("[DB] Execute: {}", sql);
                Ok(DatabaseResponse::Execute(Ok(1)))
            }
        }
    }
}

fn main() -> Result<(), String> {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    info!("=== Approach A: Macro-Generated WIT Interfaces ===");

    // Create handlers for each interface
    let mut fs_handler = FilesystemHandler::new();
    let mut net_handler = NetworkHandler;
    let mut db_handler = DatabaseHandler;

    info!("Demonstrating Filesystem Interface:");

    // Test filesystem operations
    let write_cmd = FilesystemCommands::Write {
        path: "/etc/config.toml".to_string(),
        data: b"[server]\nport = 8080\n".to_vec(),
    };
    fs_handler.execute(&write_cmd)?;

    let read_cmd = FilesystemCommands::Read {
        path: "/etc/config.toml".to_string(),
    };
    let result = fs_handler.execute(&read_cmd)?;
    if let FilesystemResponse::Read(Ok(data)) = result {
        info!("Read config: {} bytes", data.len());
    }

    info!("Demonstrating Network Interface:");

    // Test network operations
    let get_cmd = NetworkCommands::Http_get {
        url: "https://api.tairitsu.dev/status".to_string(),
    };
    let result = net_handler.execute(&get_cmd)?;
    info!("Response: {:?}", result);

    info!("Demonstrating Database Interface:");

    // Test database operations
    let query_cmd = DatabaseCommands::Query {
        sql: "SELECT * FROM users WHERE active = true".to_string(),
    };
    let result = db_handler.execute(&query_cmd)?;
    info!("Query result: {:?}", result);

    info!("=== Architecture Highlights ===");
    info!("Zero boilerplate - Macros generate all enum code from WIT syntax");
    info!("Compile-time type safety - Rust type system enforces correctness");
    info!("No runtime serialization - Direct function calls");
    info!("Three independent interfaces - Filesystem, Network, Database");
    info!("Single source of truth - WIT interface definitions drive everything");
    info!("Automatic command routing - Enum variants map to function names");
    info!("IDE integration - Full autocomplete and type hints for all commands");

    Ok(())
}
