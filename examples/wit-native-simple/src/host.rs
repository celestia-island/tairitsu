//! Approach B: Trait-based WIT integration with WASM container
//!
//! This demonstrates how to integrate the trait-based composable WIT interfaces
//! with the actual WASM container system, achieving zero serialization overhead
//! while maintaining full compile-time type safety.

use anyhow::Result;
use std::collections::HashMap;

use tairitsu::{CompositeWitInterface, WitCommand, WitCommandHandler, WitInterface};

// ============================================================================
// Define WIT-Compatible Command Types (Zero Serialization)
// ============================================================================

/// File system commands - basic operations
#[derive(Debug, Clone)]
pub enum FileSystemCommands {
    Read { path: String },
    Write { path: String, data: Vec<u8> },
    Delete { path: String },
    List { directory: String },
}

impl WitCommand for FileSystemCommands {
    type Response = Result<Vec<u8>, String>;

    fn command_name(&self) -> &'static str {
        match self {
            Self::Read { .. } => "fs_read",
            Self::Write { .. } => "fs_write",
            Self::Delete { .. } => "fs_delete",
            Self::List { .. } => "fs_list",
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Network commands demonstrating interface composition
#[derive(Debug, Clone)]
pub enum NetworkCommands {
    HttpGet { url: String },
    HttpPost { url: String, body: Vec<u8> },
}

impl WitCommand for NetworkCommands {
    type Response = Result<Vec<u8>, String>;

    fn command_name(&self) -> &'static str {
        match self {
            Self::HttpGet { .. } => "net_http_get",
            Self::HttpPost { .. } => "net_http_post",
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// ============================================================================
// Handler Implementations
// ============================================================================

/// File system handler with in-memory storage
struct FileSystemHandler {
    storage: HashMap<String, Vec<u8>>,
}

impl FileSystemHandler {
    fn new() -> Self {
        Self {
            storage: HashMap::new(),
        }
    }
}

impl WitCommandHandler<FileSystemCommands> for FileSystemHandler {
    fn execute(
        &mut self,
        command: &FileSystemCommands,
    ) -> Result<<FileSystemCommands as WitCommand>::Response, String> {
        match command {
            FileSystemCommands::Read { path } => self
                .storage
                .get(path)
                .cloned()
                .map(Ok)
                .ok_or_else(|| format!("File not found: {}", path)),
            FileSystemCommands::Write { path, data } => {
                self.storage.insert(path.clone(), data.clone());
                Ok(Ok(b"Written successfully".to_vec()))
            }
            FileSystemCommands::Delete { path } => self
                .storage
                .remove(path)
                .map(|_| Ok(b"Deleted successfully".to_vec()))
                .ok_or_else(|| format!("File not found: {}", path)),
            FileSystemCommands::List { directory } => {
                let files: Vec<String> = self
                    .storage
                    .keys()
                    .filter(|k| k.starts_with(directory))
                    .cloned()
                    .collect();
                let files_json = files.join(",");
                Ok(Ok(files_json.into_bytes()))
            }
        }
    }
}

/// Network handler (mock implementation)
struct NetworkHandler;

impl WitCommandHandler<NetworkCommands> for NetworkHandler {
    fn execute(
        &mut self,
        command: &NetworkCommands,
    ) -> Result<<NetworkCommands as WitCommand>::Response, String> {
        match command {
            NetworkCommands::HttpGet { url } => {
                // Mock HTTP GET
                Ok(Ok(format!("GET response from {}", url).into_bytes()))
            }
            NetworkCommands::HttpPost { url, body } => {
                // Mock HTTP POST
                Ok(Ok(
                    format!("POST to {} with {} bytes", url, body.len()).into_bytes()
                ))
            }
        }
    }
}

// ============================================================================
// WIT Interface Implementations
// ============================================================================

#[allow(dead_code)]
struct FileSystemInterface {
    handler: FileSystemHandler,
}

impl FileSystemInterface {
    fn new() -> Self {
        Self {
            handler: FileSystemHandler::new(),
        }
    }

    #[allow(dead_code)]
    fn handler_mut(&mut self) -> &mut FileSystemHandler {
        &mut self.handler
    }
}

impl WitInterface for FileSystemInterface {
    fn interface_name(&self) -> &'static str {
        "filesystem"
    }

    fn register_handlers(&self, _dispatcher: &mut tairitsu::WitCommandDispatcher) {
        println!("[Interface] Registered: {}", self.interface_name());
    }
}

struct NetworkInterface;

impl WitInterface for NetworkInterface {
    fn interface_name(&self) -> &'static str {
        "network"
    }

    fn register_handlers(&self, _dispatcher: &mut tairitsu::WitCommandDispatcher) {
        println!("[Interface] Registered: {}", self.interface_name());
    }
}

// ============================================================================
// Main: Demonstrating Trait-Based WIT Integration
// ============================================================================

fn main() -> Result<()> {
    println!("=== Approach B: Trait-Based WIT Integration with Container ===\n");

    // Step 1: Create composite interface combining multiple WIT implementations
    let mut composite = CompositeWitInterface::new();

    println!("Step 1: Building composite interface");
    composite.add_interface(Box::new(FileSystemInterface::new()));
    composite.add_interface(Box::new(NetworkInterface));
    println!("  ✓ Added filesystem interface");
    println!("  ✓ Added network interface\n");

    // Step 2: Create dispatcher and register all handlers
    let mut dispatcher = tairitsu::WitCommandDispatcher::new();
    composite.register_all(&mut dispatcher);

    // Step 3: Demonstrate zero-serialization command execution
    println!("Step 2: Executing commands with zero serialization overhead\n");

    // Create handler instances (in real implementation, these would be in the container)
    let mut fs_handler = FileSystemHandler::new();
    let mut net_handler = NetworkHandler;

    // File system operations
    println!("--- File System Operations ---");

    let write_cmd = FileSystemCommands::Write {
        path: "/data/config.json".to_string(),
        data: b"{\"name\":\"tairitsu\",\"version\":\"0.1.0\"}".to_vec(),
    };
    println!("Command: {:?}", write_cmd);
    match fs_handler
        .execute(&write_cmd)
        .map_err(|e| anyhow::anyhow!(e))?
    {
        Ok(result) => println!("  ✓ Result: {}\n", String::from_utf8_lossy(&result)),
        Err(e) => println!("  ✗ Error: {}\n", e),
    }

    let read_cmd = FileSystemCommands::Read {
        path: "/data/config.json".to_string(),
    };
    println!("Command: {:?}", read_cmd);
    match fs_handler
        .execute(&read_cmd)
        .map_err(|e| anyhow::anyhow!(e))?
    {
        Ok(data) => {
            let content = String::from_utf8_lossy(&data);
            println!("  ✓ Read: {}\n", content);
        }
        Err(e) => println!("  ✗ Error: {}\n", e),
    }

    // Write more files
    let _ = fs_handler
        .execute(&FileSystemCommands::Write {
            path: "/data/file1.txt".to_string(),
            data: b"Content 1".to_vec(),
        })
        .map_err(|e| anyhow::anyhow!(e))?;
    let _ = fs_handler
        .execute(&FileSystemCommands::Write {
            path: "/data/file2.txt".to_string(),
            data: b"Content 2".to_vec(),
        })
        .map_err(|e| anyhow::anyhow!(e))?;

    let list_cmd = FileSystemCommands::List {
        directory: "/data".to_string(),
    };
    println!("Command: {:?}", list_cmd);
    match fs_handler
        .execute(&list_cmd)
        .map_err(|e| anyhow::anyhow!(e))?
    {
        Ok(files) => {
            let files_str = String::from_utf8_lossy(&files);
            println!("  ✓ Files: {}\n", files_str);
        }
        Err(e) => println!("  ✗ Error: {}\n", e),
    }

    // Network operations
    println!("--- Network Operations ---");

    let get_cmd = NetworkCommands::HttpGet {
        url: "https://api.example.com/data".to_string(),
    };
    println!("Command: {:?}", get_cmd);
    match net_handler
        .execute(&get_cmd)
        .map_err(|e| anyhow::anyhow!(e))?
    {
        Ok(response) => println!("  ✓ Response: {}\n", String::from_utf8_lossy(&response)),
        Err(e) => println!("  ✗ Error: {}\n", e),
    }

    let post_cmd = NetworkCommands::HttpPost {
        url: "https://api.example.com/submit".to_string(),
        body: b"{\"action\":\"test\"}".to_vec(),
    };
    println!("Command: {:?}", post_cmd);
    match net_handler
        .execute(&post_cmd)
        .map_err(|e| anyhow::anyhow!(e))?
    {
        Ok(response) => println!("  ✓ Response: {}\n", String::from_utf8_lossy(&response)),
        Err(e) => println!("  ✗ Error: {}\n", e),
    }

    // Summary
    println!("=== Architecture Benefits ===");
    println!("✓ Zero serialization overhead - Direct function calls");
    println!("✓ Compile-time type safety - Rust type system enforces correctness");
    println!("✓ Composable interfaces - Multiple WIT implementations combined");
    println!("✓ Interface extension - Handlers can build on each other");
    println!("✓ Dynamic dispatch - trait objects enable runtime flexibility");
    println!("\n=== Integration with WASM Container ===");
    println!("This approach can be integrated with the Container system by:");
    println!("1. Container maintains a CompositeWitInterface");
    println!("2. WASM guest calls host functions via WIT bindgen");
    println!("3. Host dispatches to appropriate trait handlers");
    println!("4. Responses flow back through WIT without serialization");
    println!("\nFor WASM integration example, see: examples/hybrid");

    Ok(())
}
