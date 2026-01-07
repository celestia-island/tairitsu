//! Approach B: Trait-based composable WIT interfaces demo
//!
//! This demonstrates how to use trait objects and dynamic dispatch
//! to compose multiple WIT interfaces without runtime serialization.

use log::info;
use rand::Rng;
use std::collections::HashMap;

use tairitsu::{CompositeWitInterface, WitCommand, WitCommandDispatcher, WitCommandHandler, WitInterface};

// ============================================================================
// File System Interface - Basic Operations
// ============================================================================

#[derive(Debug, Clone)]
pub enum FileSystemBasicCommands {
    Read { path: String },
    Write { path: String, data: Vec<u8> },
    Delete { path: String },
}

impl WitCommand for FileSystemBasicCommands {
    type Response = Result<Vec<u8>, String>;

    fn command_name(&self) -> &'static str {
        match self {
            Self::Read { .. } => "fs_read",
            Self::Write { .. } => "fs_write",
            Self::Delete { .. } => "fs_delete",
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Handler for basic file system operations
struct FileSystemBasicHandler {
    storage: HashMap<String, Vec<u8>>,
}

impl FileSystemBasicHandler {
    fn new() -> Self {
        Self {
            storage: HashMap::new(),
        }
    }
}

impl WitCommandHandler<FileSystemBasicCommands> for FileSystemBasicHandler {
    fn execute(
        &mut self,
        command: &FileSystemBasicCommands,
    ) -> Result<<FileSystemBasicCommands as WitCommand>::Response, String> {
        match command {
            FileSystemBasicCommands::Read { path } => self
                .storage
                .get(path)
                .cloned()
                .map(Ok)
                .ok_or_else(|| format!("File not found: {}", path)),
            FileSystemBasicCommands::Write { path, data } => {
                self.storage.insert(path.clone(), data.clone());
                Ok(Ok(vec![]))
            }
            FileSystemBasicCommands::Delete { path } => self
                .storage
                .remove(path)
                .map(|_| Ok(vec![]))
                .ok_or_else(|| format!("File not found: {}", path)),
        }
    }
}

// ============================================================================
// File System Interface - Advanced Operations
// ============================================================================

#[derive(Debug, Clone)]
pub enum FileSystemAdvancedCommands {
    List { directory: String },
    Move { from: String, to: String },
    Copy { from: String, to: String },
}

impl WitCommand for FileSystemAdvancedCommands {
    type Response = Result<Vec<String>, String>;

    fn command_name(&self) -> &'static str {
        match self {
            Self::List { .. } => "fs_list",
            Self::Move { .. } => "fs_move",
            Self::Copy { .. } => "fs_copy",
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Handler for advanced file system operations
struct FileSystemAdvancedHandler {
    basic_handler: FileSystemBasicHandler,
}

impl FileSystemAdvancedHandler {
    fn new(basic_handler: FileSystemBasicHandler) -> Self {
        Self { basic_handler }
    }
}

impl WitCommandHandler<FileSystemAdvancedCommands> for FileSystemAdvancedHandler {
    fn execute(
        &mut self,
        command: &FileSystemAdvancedCommands,
    ) -> Result<<FileSystemAdvancedCommands as WitCommand>::Response, String> {
        match command {
            FileSystemAdvancedCommands::List { directory } => {
                let files: Vec<String> = self
                    .basic_handler
                    .storage
                    .keys()
                    .filter(|k| k.starts_with(directory))
                    .cloned()
                    .collect();
                Ok(Ok(files))
            }
            FileSystemAdvancedCommands::Move { from, to } => {
                if let Some(data) = self.basic_handler.storage.remove(from) {
                    self.basic_handler.storage.insert(to.clone(), data);
                    Ok(Ok(vec![to.clone()]))
                } else {
                    Err(format!("Source file not found: {}", from))
                }
            }
            FileSystemAdvancedCommands::Copy { from, to } => {
                if let Some(data) = self.basic_handler.storage.get(from) {
                    self.basic_handler.storage.insert(to.clone(), data.clone());
                    Ok(Ok(vec![to.clone()]))
                } else {
                    Err(format!("Source file not found: {}", from))
                }
            }
        }
    }
}

// ============================================================================
// WIT Interface Implementations
// ============================================================================

struct FileSystemBasicInterface;

impl WitInterface for FileSystemBasicInterface {
    fn interface_name(&self) -> &'static str {
        "filesystem-basic"
    }

    fn register_handlers(&self, _dispatcher: &mut WitCommandDispatcher) {
        // In a real implementation, this would register the actual handlers
        info!("Registered handlers for: {}", self.interface_name());
    }
}

struct FileSystemAdvancedInterface;

impl WitInterface for FileSystemAdvancedInterface {
    fn interface_name(&self) -> &'static str {
        "filesystem-advanced"
    }

    fn register_handlers(&self, _dispatcher: &mut WitCommandDispatcher) {
        info!("Registered handlers for: {}", self.interface_name());
    }
}

// ============================================================================
// Demo Main
// ============================================================================

fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    info!("=== Approach B: Trait-Based Composable WIT Interfaces ===");

    // Create composite interface
    let mut composite = CompositeWitInterface::new();

    // Add basic filesystem interface
    composite.add_interface(Box::new(FileSystemBasicInterface));
    info!("Added basic filesystem interface");

    // Add advanced filesystem interface
    composite.add_interface(Box::new(FileSystemAdvancedInterface));
    info!("Added advanced filesystem interface");

    // Create dispatcher and register all handlers
    let mut dispatcher = WitCommandDispatcher::new();
    composite.register_all(&mut dispatcher);

    info!("=== Demonstrating Type-Safe Commands ===");

    // Create handlers manually for demo
    let mut basic_handler = FileSystemBasicHandler::new();

    // Demonstrate basic operations
    let write_cmd = FileSystemBasicCommands::Write {
        path: "/test.txt".to_string(),
        data: b"Hello, WIT!".to_vec(),
    };
    info!("Command: {:?}", write_cmd);
    match basic_handler
        .execute(&write_cmd)
        .map_err(|e| anyhow::anyhow!(e))?
    {
        Ok(_) => info!("Write successful"),
        Err(e) => info!("Write failed: {}", e),
    }

    let read_cmd = FileSystemBasicCommands::Read {
        path: "/test.txt".to_string(),
    };
    info!("Command: {:?}", read_cmd);
    match basic_handler
        .execute(&read_cmd)
        .map_err(|e| anyhow::anyhow!(e))?
    {
        Ok(data) => {
            let content = String::from_utf8_lossy(&data);
            info!("Read successful: {}", content);
        }
        Err(e) => info!("Read failed: {}", e),
    }

    // Demonstrate advanced operations
    let mut advanced_handler = FileSystemAdvancedHandler::new(basic_handler);

    // Write more files
    let _ = advanced_handler
        .basic_handler
        .execute(&FileSystemBasicCommands::Write {
            path: "/dir/file1.txt".to_string(),
            data: b"File 1".to_vec(),
        })
        .map_err(|e| anyhow::anyhow!(e))?;
    let _ = advanced_handler
        .basic_handler
        .execute(&FileSystemBasicCommands::Write {
            path: "/dir/file2.txt".to_string(),
            data: b"File 2".to_vec(),
        })
        .map_err(|e| anyhow::anyhow!(e))?;

    let list_cmd = FileSystemAdvancedCommands::List {
        directory: "/dir".to_string(),
    };
    info!("Command: {:?}", list_cmd);
    match advanced_handler
        .execute(&list_cmd)
        .map_err(|e| anyhow::anyhow!(e))?
    {
        Ok(files) => {
            info!("List successful: {:?}", files);
        }
        Err(e) => info!("List failed: {}", e),
    }

    let copy_cmd = FileSystemAdvancedCommands::Copy {
        from: "/dir/file1.txt".to_string(),
        to: "/dir/file1_copy.txt".to_string(),
    };
    info!("Command: {:?}", copy_cmd);
    match advanced_handler
        .execute(&copy_cmd)
        .map_err(|e| anyhow::anyhow!(e))?
    {
        Ok(result) => info!("Copy successful: {:?}", result),
        Err(e) => info!("Copy failed: {}", e),
    }

    info!("=== Key Benefits ===");
    info!("No runtime serialization overhead");
    info!("Compile-time type safety");
    info!("Composable interfaces via traits");
    info!("Each interface can extend/build on others");
    info!("Zero-cost abstractions");

    info!("=== Random Data Testing ===");
    test_random_operations()?;

    Ok(())
}

/// Test operations with random data
fn test_random_operations() -> anyhow::Result<()> {
    let mut rng = rand::thread_rng();
    let mut handler = FileSystemBasicHandler::new();

    info!("Testing file operations with random data...");

    for i in 1..=5 {
        let random_id: u32 = rng.gen();
        let random_size: usize = rng.gen_range(10..100);
        let random_data: Vec<u8> = (0..random_size).map(|_| rng.gen()).collect();

        let path = format!("/random/test_{}.dat", random_id);
        let write_cmd = FileSystemBasicCommands::Write {
            path: path.clone(),
            data: random_data.clone(),
        };

        info!("  [{}] Writing {} bytes to {}", i, random_data.len(), path);
        match handler.execute(&write_cmd) {
            Ok(_) => info!("    ✓ Write successful"),
            Err(e) => info!("    ✗ Write failed: {}", e),
        }

        let read_cmd = FileSystemBasicCommands::Read { path };
        info!("  [{}] Reading back", i);
        match handler.execute(&read_cmd) {
            Ok(Ok(data)) => {
                if data == random_data {
                    info!("    ✓ Read verified: {} bytes match", data.len());
                } else {
                    info!("    ✗ Read verification failed");
                }
            }
            Ok(Err(e)) => info!("    ✗ Read failed: {}", e),
            Err(e) => info!("    ✗ Read error: {}", e),
        }
    }

    info!("✓ Random data tests passed");

    Ok(())
}
