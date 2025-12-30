//! Host side implementation - runs natively
//! This demonstrates how the host can load WASM modules and communicate bidirectionally

use anyhow::Result;
use bytes::Bytes;
use tairitsu::{Container, GuestCommands, HostCommands, HostResponse, LogLevel, Registry};

fn main() -> Result<()> {
    println!("=== Tairitsu Hybrid Example - Host Side (Type-Safe Commands) ===\n");

    // Load the WASM module (the same crate compiled to wasm32-wasip1)
    let wasm_path = format!(
        "{}/../../target/wasm32-wasip1/release/tairitsu_example_hybrid.wasm",
        env!("CARGO_MANIFEST_DIR")
    );

    println!("Loading WASM module from: {}", wasm_path);
    let wasm_binary = std::fs::read(&wasm_path)
        .map(Bytes::from)
        .inspect_err(|_e| {
            eprintln!("\nError: Could not load WASM file.");
            eprintln!("Please build the WASM module first with:");
            eprintln!(
                "  cargo build --target wasm32-wasip1 --release --package tairitsu-example-hybrid"
            );
        })?;

    println!("WASM module loaded ({} bytes)\n", wasm_binary.len());

    // Create a registry (like a Docker daemon)
    let registry = Registry::new();

    // Register the WASM binary as an image
    println!("Registering WASM module as image 'hybrid-example:latest'");
    registry.register_image("hybrid-example:latest", wasm_binary)?;

    // Get the image
    let image = registry
        .get_image("hybrid-example:latest")
        .expect("Image should be registered");

    // Create a container from the image
    println!("Creating container from image...\n");
    let mut container = Container::new(&image)?;

    // Set up host-side handlers with typed commands
    container.on_execute(|command: HostCommands| {
        println!("[Host] Received execute request: {:?}", command);
        match command {
            HostCommands::GetInfo => Ok(HostResponse::Info {
                name: "Tairitsu Host".to_string(),
                version: "0.1.0".to_string(),
                status: "running".to_string(),
            }),
            HostCommands::Echo(msg) => Ok(HostResponse::Text(msg)),
            HostCommands::Custom { name, data } => {
                // Fallback for legacy commands
                Ok(HostResponse::Text(format!(
                    "Custom command '{}' with data: {}",
                    name, data
                )))
            }
        }
    });

    container.on_log(|level: LogLevel, message: String| {
        println!(
            "[Guest Log][{}] {}",
            level.to_string().to_uppercase(),
            message
        );
    });

    // Initialize the guest module
    println!("=== Initializing Guest Module ===");
    container.init()?;

    println!("\n=== Sending Typed Commands to Guest ===");

    // Send type-safe commands to the guest
    let commands = vec![
        GuestCommands::Greet("Tairitsu Framework".to_string()),
        GuestCommands::Compute("Hello World".to_string()),
        GuestCommands::CallHost("This message goes to host and back".to_string()),
    ];

    for cmd in commands {
        println!("\n[Host] Sending typed command: {:?}", cmd);
        match container.send_command(cmd) {
            Ok(response) => {
                println!("[Host] Guest response: {:?}", response);
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
    println!("3. Type-safe bidirectional communication:");
    println!("   - Host calling Guest with enum-based commands");
    println!("   - Guest calling Host with enum-based commands");
    println!("   - Compile-time type safety for all commands and responses");
    println!("4. Shared enum definitions for type-safe communication");

    Ok(())
}
