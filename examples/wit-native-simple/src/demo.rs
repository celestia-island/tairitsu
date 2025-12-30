//! Approach B: Trait-based composable WIT interfaces demo
//!
//! This demonstrates how to use trait objects and dynamic dispatch
//! to compose multiple WIT interfaces without runtime serialization.

use std::collections::HashMap;
use tairitsu::{
    CompositeWitInterface, WitCommand, WitCommandDispatcher, WitCommandHandler, WitInterface,
};

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
        println!("Registered handlers for: {}", self.interface_name());
    }
}

struct FileSystemAdvancedInterface;

impl WitInterface for FileSystemAdvancedInterface {
    fn interface_name(&self) -> &'static str {
        "filesystem-advanced"
    }

    fn register_handlers(&self, _dispatcher: &mut WitCommandDispatcher) {
        println!("Registered handlers for: {}", self.interface_name());
    }
}

// ============================================================================
// Demo Main
// ============================================================================

fn main() -> anyhow::Result<()> {
    println!("=== Approach B: Trait-Based Composable WIT Interfaces ===\n");

    // Create composite interface
    let mut composite = CompositeWitInterface::new();

    // Add basic filesystem interface
    composite.add_interface(Box::new(FileSystemBasicInterface));
    println!("✓ Added basic filesystem interface");

    // Add advanced filesystem interface
    composite.add_interface(Box::new(FileSystemAdvancedInterface));
    println!("✓ Added advanced filesystem interface");

    // Create dispatcher and register all handlers
    let mut dispatcher = WitCommandDispatcher::new();
    composite.register_all(&mut dispatcher);

    println!("\n=== Demonstrating Type-Safe Commands ===\n");

    // Create handlers manually for demo
    let mut basic_handler = FileSystemBasicHandler::new();

    // Demonstrate basic operations
    let write_cmd = FileSystemBasicCommands::Write {
        path: "/test.txt".to_string(),
        data: b"Hello, WIT!".to_vec(),
    };
    println!("Command: {:?}", write_cmd);
    match basic_handler
        .execute(&write_cmd)
        .map_err(|e| anyhow::anyhow!(e))?
    {
        Ok(_) => println!("✓ Write successful\n"),
        Err(e) => println!("✗ Write failed: {}\n", e),
    }

    let read_cmd = FileSystemBasicCommands::Read {
        path: "/test.txt".to_string(),
    };
    println!("Command: {:?}", read_cmd);
    match basic_handler
        .execute(&read_cmd)
        .map_err(|e| anyhow::anyhow!(e))?
    {
        Ok(data) => {
            let content = String::from_utf8_lossy(&data);
            println!("✓ Read successful: {}\n", content);
        }
        Err(e) => println!("✗ Read failed: {}\n", e),
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
    println!("Command: {:?}", list_cmd);
    match advanced_handler
        .execute(&list_cmd)
        .map_err(|e| anyhow::anyhow!(e))?
    {
        Ok(files) => {
            println!("✓ List successful: {:?}\n", files);
        }
        Err(e) => println!("✗ List failed: {}\n", e),
    }

    let copy_cmd = FileSystemAdvancedCommands::Copy {
        from: "/dir/file1.txt".to_string(),
        to: "/dir/file1_copy.txt".to_string(),
    };
    println!("Command: {:?}", copy_cmd);
    match advanced_handler
        .execute(&copy_cmd)
        .map_err(|e| anyhow::anyhow!(e))?
    {
        Ok(result) => println!("✓ Copy successful: {:?}\n", result),
        Err(e) => println!("✗ Copy failed: {}\n", e),
    }

    println!("=== Key Benefits ===");
    println!("✓ No runtime serialization overhead");
    println!("✓ Compile-time type safety");
    println!("✓ Composable interfaces via traits");
    println!("✓ Each interface can extend/build on others");
    println!("✓ Zero-cost abstractions");

    Ok(())
}
