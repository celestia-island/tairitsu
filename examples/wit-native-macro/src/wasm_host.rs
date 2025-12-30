//! Approach A: WASM Host - Loading and communicating with macro-generated WASM guest
//! This demonstrates the full bidirectional communication cycle with type-safe commands

use anyhow::Result;
use std::path::PathBuf;

use tairitsu::{Container, GuestCommands, HostCommands, HostResponse, LogLevel, Registry};

fn main() -> Result<()> {
    println!("=== Approach A: Macro-Generated Type-Safe WASM Communication ===\n");

    // Build the WASM path
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let wasm_path =
        manifest_dir.join("../../target/wasm32-wasip1/release/tairitsu_example_wit_native_a.wasm");

    println!("Loading WASM module from: {}", wasm_path.display());

    // Load the WASM binary
    let wasm_binary = std::fs::read(&wasm_path).map_err(|e| {
        eprintln!("\nError: Could not load WASM file: {}", e);
        eprintln!("Please build the WASM module first with:");
        eprintln!("  cargo build --target wasm32-wasip1 --release --package tairitsu-example-wit-native-a --lib");
        e
    })?;

    println!("WASM module loaded ({} bytes)\n", wasm_binary.len());

    // Create registry and register the WASM module
    let registry = Registry::new();
    registry.register_image("wit-native-a:latest", wasm_binary.into())?;

    // Get the image and create a container
    let image = registry
        .get_image("wit-native-a:latest")
        .expect("Image should be registered");

    println!("Creating container from image...\n");
    let mut container = Container::new(&image)?;

    // Set up host-side handlers with macro-generated type-safe commands
    container.on_execute(|command: HostCommands| {
        println!("[Host] Received execute request: {:?}", command);
        match command {
            HostCommands::GetInfo => Ok(HostResponse::Info {
                name: "Tairitsu Host (Approach A)".to_string(),
                version: "0.1.0".to_string(),
                status: "running with macro-generated commands".to_string(),
            }),
            HostCommands::Echo(msg) => {
                println!("[Host] Echoing message: {}", msg);
                Ok(HostResponse::Text(msg))
            }
            HostCommands::Custom { name, data } => Ok(HostResponse::Text(format!(
                "Custom command '{}' with data: {}",
                name, data
            ))),
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

    // Send type-safe commands to the guest (using macro-generated enums)
    let commands = vec![
        GuestCommands::Greet("Tairitsu with Approach A".to_string()),
        GuestCommands::Compute("The quick brown fox jumps over the lazy dog".to_string()),
        GuestCommands::CallHost("Test nested bidirectional call".to_string()),
    ];

    for cmd in commands {
        println!("\n[Host] Sending macro-generated typed command: {:?}", cmd);
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
    println!("\nApproach A demonstrated:");
    println!("  ✓ Macro-generated type-safe command enums");
    println!("  ✓ Zero boilerplate - WIT defines everything");
    println!("  ✓ Host → Guest bidirectional communication");
    println!("  ✓ Guest → Host bidirectional communication");
    println!("  ✓ Nested calls (Guest calling Host during Host's request)");
    println!("  ✓ Compile-time type safety across WASM boundary");

    Ok(())
}
