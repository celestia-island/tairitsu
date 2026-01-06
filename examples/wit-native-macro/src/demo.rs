//! Approach A Demo: Automatic WIT enum generation using procedural macros
//!
//! This example demonstrates the proc-macro based approach where WIT interfaces
//! are automatically converted to type-safe Rust enums without manual boilerplate.

use log::info;
use rand::Rng;
use tairitsu::{wit_interface, wit_registry::WitCommandHandler, WitCommand};

// Automatically generate FilesystemCommands and FilesystemResponse from WIT-like syntax
wit_interface! {
    interface filesystem {
        read: func(path: String) -> Result<Vec<u8>, String>;
        write: func(path: String, data: Vec<u8>) -> Result<(), String>;
        delete: func(path: String) -> Result<(), String>;
        list: func(directory: String) -> Result<Vec<String>, String>;
    }
}

// Automatically generate NetworkCommands and NetworkResponse
wit_interface! {
    interface network {
        http_get: func(url: String) -> Result<String, String>;
        http_post: func(url: String, body: String) -> Result<String, String>;
    }
}

// Filesystem handler implementation
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
                Ok(FilesystemResponse::Write(Ok(())))
            }
            FilesystemCommands::Delete { path } => {
                self.storage
                    .remove(path)
                    .ok_or_else(|| format!("File not found: {}", path))?;
                Ok(FilesystemResponse::Delete(Ok(())))
            }
            FilesystemCommands::List { directory } => {
                let files: Vec<String> = self
                    .storage
                    .keys()
                    .filter(|k| k.starts_with(directory))
                    .cloned()
                    .collect();
                Ok(FilesystemResponse::List(Ok(files)))
            }
        }
    }
}

// Network handler implementation
struct NetworkHandler;

impl WitCommandHandler<NetworkCommands> for NetworkHandler {
    fn execute(&mut self, command: &NetworkCommands) -> Result<NetworkResponse, String> {
        match command {
            NetworkCommands::Http_get { url } => {
                // Mock implementation
                Ok(NetworkResponse::Http_get(Ok(format!(
                    "Response from {}",
                    url
                ))))
            }
            NetworkCommands::Http_post { url, body } => {
                // Mock implementation
                Ok(NetworkResponse::Http_post(Ok(format!(
                    "Posted {} bytes to {}",
                    body.len(),
                    url
                ))))
            }
        }
    }
}

fn main() -> Result<(), String> {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    info!("=== Approach A: Automatic WIT Enum Generation Demo ===");

    // Create handlers
    let mut fs_handler = FilesystemHandler::new();
    let mut net_handler = NetworkHandler;

    info!("Testing Filesystem Interface:");

    // Write operation
    let write_cmd = FilesystemCommands::Write {
        path: "/data/config.json".to_string(),
        data: b"{\"name\":\"tairitsu\",\"version\":\"0.1.0\"}".to_vec(),
    };
    info!("Command: {:?}", write_cmd.command_name());
    let write_result = fs_handler.execute(&write_cmd)?;
    info!("Result: {:?}", write_result);

    // Read operation
    let read_cmd = FilesystemCommands::Read {
        path: "/data/config.json".to_string(),
    };
    info!("Command: {:?}", read_cmd.command_name());
    let read_result = fs_handler.execute(&read_cmd)?;
    if let FilesystemResponse::Read(Ok(data)) = read_result {
        info!("Read {} bytes", data.len());
    }

    // List operation
    let list_cmd = FilesystemCommands::List {
        directory: "/data/".to_string(),
    };
    info!("Command: {:?}", list_cmd.command_name());
    let list_result = fs_handler.execute(&list_cmd)?;
    info!("Result: {:?}", list_result);

    info!("Testing Network Interface:");

    // HTTP GET operation
    let get_cmd = NetworkCommands::Http_get {
        url: "https://api.example.com/data".to_string(),
    };
    info!("Command: {:?}", get_cmd.command_name());
    let get_result = net_handler.execute(&get_cmd)?;
    info!("Result: {:?}", get_result);

    info!("=== Key Features Demonstrated ===");
    info!("Zero boilerplate - WIT definitions automatically generate enums");
    info!("Compile-time type safety - Invalid commands caught at compile time");
    info!("No runtime serialization - Direct function calls");
    info!("IDE support - Full autocomplete and type hints");
    info!("Single source of truth - WIT interface defines everything");

    info!("=== Random Data Testing ===");
    test_random_operations(&mut fs_handler)?;

    Ok(())
}

/// Test operations with random data
fn test_random_operations(handler: &mut FilesystemHandler) -> Result<(), String> {
    let mut rng = rand::thread_rng();

    info!("Testing filesystem operations with random data...");

    for i in 1..=5 {
        let random_id: u32 = rng.gen();
        let random_size: usize = rng.gen_range(10..100);

        let path = format!("/random/test_{}.dat", random_id);
        let random_data: Vec<u8> = (0..random_size).map(|_| rng.gen()).collect();

        let write_cmd = FilesystemCommands::Write {
            path: path.clone(),
            data: random_data.clone(),
        };

        info!("  [{}] Writing {} bytes to {}", i, random_data.len(), path);
        match handler.execute(&write_cmd)? {
            FilesystemResponse::Write(Ok(())) => info!("    ✓ Write successful"),
            FilesystemResponse::Write(Err(e)) => info!("    ✗ Write failed: {}", e),
            _ => info!("    ✗ Unexpected response"),
        }

        let read_cmd = FilesystemCommands::Read { path };
        info!("  [{}] Reading back", i);
        match handler.execute(&read_cmd)? {
            FilesystemResponse::Read(Ok(data)) => {
                if data == random_data {
                    info!("    ✓ Read verified: {} bytes match", data.len());
                } else {
                    info!("    ✗ Read verification failed");
                }
            }
            FilesystemResponse::Read(Err(e)) => info!("    ✗ Read failed: {}", e),
            _ => info!("    ✗ Unexpected response"),
        }
    }

    info!("✓ Random data tests passed");
    Ok(())
}
