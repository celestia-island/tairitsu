//! Host side implementation - runs natively
//! This demonstrates how the host can load WASM modules and communicate bidirectionally

use anyhow::Result;
use bytes::Bytes;
use tairitsu::{Container, Registry};

fn main() -> Result<()> {
    println!("=== Tairitsu Hybrid Example - Host Side ===\n");
    
    // Load the WASM module (the same crate compiled to wasm32-wasip1)
    let wasm_path = format!(
        "{}/../../target/wasm32-wasip1/release/tairitsu_example_hybrid.wasm",
        env!("CARGO_MANIFEST_DIR")
    );
    
    println!("Loading WASM module from: {}", wasm_path);
    let wasm_binary = std::fs::read(&wasm_path)
        .map(Bytes::from)
        .map_err(|e| {
            eprintln!("\nError: Could not load WASM file.");
            eprintln!("Please build the WASM module first with:");
            eprintln!("  cargo build --target wasm32-wasip1 --release --package tairitsu-example-hybrid");
            e
        })?;
    
    println!("WASM module loaded ({} bytes)\n", wasm_binary.len());
    
    // Create a registry (like a Docker daemon)
    let registry = Registry::new();
    
    // Register the WASM binary as an image
    println!("Registering WASM module as image 'hybrid-example:latest'");
    registry.register_image("hybrid-example:latest", wasm_binary)?;
    
    // Get the image
    let image = registry.get_image("hybrid-example:latest")
        .expect("Image should be registered");
    
    // Create a container from the image
    println!("Creating container from image...\n");
    let mut container = Container::new(&image)?;
    
    // Set up host-side handlers
    container.on_execute(|command: String, payload: String| {
        println!("[Host] Received execute request: command='{}', payload='{}'", command, payload);
        match command.as_str() {
            "get_info" => {
                Ok(r#"{"name":"Tairitsu Host","version":"0.1.0","status":"running"}"#.to_string())
            }
            "echo" => {
                Ok(payload)
            }
            _ => {
                Err(format!("Unknown host command: {}", command))
            }
        }
    });
    
    container.on_log(|level: String, message: String| {
        println!("[Guest Log][{}] {}", level.to_uppercase(), message);
    });
    
    // Initialize the guest module
    println!("=== Initializing Guest Module ===");
    container.init()?;
    
    println!("\n=== Sending Commands to Guest ===");
    
    // Send various commands to the guest
    let commands = vec![
        ("greet", "Tairitsu Framework"),
        ("compute", "Hello World"),
        ("call_host", "This message goes to host and back"),
    ];
    
    for (cmd, payload) in commands {
        println!("\n[Host] Sending command: '{}' with payload: '{}'", cmd, payload);
        match container.handle_command(cmd, payload) {
            Ok(response) => {
                println!("[Host] Guest response: {}", response);
            }
            Err(e) => {
                eprintln!("[Host] Guest error: {}", e);
            }
        }
    }
    
    println!("\n=== Example Complete ===");
    println!("\nThis example demonstrated:");
    println!("1. Loading a WASM module into an Image (like docker pull/build)");
    println!("2. Creating a Container from the Image (like docker run)");
    println!("3. Bidirectional communication:");
    println!("   - Host calling Guest (via handle_command)");
    println!("   - Guest calling Host (via execute API)");
    println!("4. Shared WIT interface definitions for type-safe communication");
    
    Ok(())
}
