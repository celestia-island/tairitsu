//! Approach B: WASM Host - Loading and communicating with trait-based WASM guest
//! This demonstrates the full bidirectional communication cycle with composable trait-based commands

use anyhow::Result;
use std::path::PathBuf;
use tairitsu::{Container, GuestCommands, HostCommands, HostResponse, LogLevel, Registry};

fn main() -> Result<()> {
    println!("=== Approach B: Trait-Based Composable Type-Safe WASM Communication ===\n");

    // Build the WASM path
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let wasm_path = manifest_dir
        .join("../../target/wasm32-wasip1/release/tairitsu_example_wit_native_b.wasm");

    println!("Loading WASM module from: {}", wasm_path.display());

    // Load the WASM binary
    let wasm_binary = std::fs::read(&wasm_path).map_err(|e| {
        eprintln!("\nError: Could not load WASM file: {}", e);
        eprintln!("Please build the WASM module first with:");
        eprintln!("  cargo build --target wasm32-wasip1 --release --package tairitsu-example-wit-native-b --lib");
        e
    })?;

    println!("WASM module loaded ({} bytes)\n", wasm_binary.len());

    // Create registry and register the WASM module
    let registry = Registry::new();
    registry.register_image("wit-native-b:latest", wasm_binary.into())?;

    // Get the image and create a container
    let image = registry
        .get_image("wit-native-b:latest")
        .expect("Image should be registered");

    println!("Creating container from image...\n");
    let mut container = Container::new(&image)?;

    // Set up host-side handlers with trait-based composable type-safe commands
    container.on_execute(|command: HostCommands| {
        println!("[Host] Received execute request: {:?}", command);
        match command {
            HostCommands::GetInfo => Ok(HostResponse::Info {
                name: "Tairitsu Host (Approach B)".to_string(),
                version: "0.1.0".to_string(),
                status: "running with trait-based composable commands".to_string(),
            }),
            HostCommands::Echo(msg) => {
                println!("[Host] Echoing message: {}", msg);
                Ok(HostResponse::Text(msg))
            }
            HostCommands::Custom { name, data } => {
                // Handle composable trait-based custom commands
                match name.as_str() {
                    "network_get" => {
                        println!(
                            "[Host] Processing network request via trait-based interface: {}",
                            data
                        );
                        Ok(HostResponse::Text(format!(
                            "Response from {} (via composable trait interface)",
                            data
                        )))
                    }
                    _ => Ok(HostResponse::Text(format!(
                        "Custom command '{}' with data: {}",
                        name, data
                    ))),
                }
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

    // Send type-safe commands to the guest (using trait-based composable enums)
    let commands = vec![
        GuestCommands::Greet("Tairitsu with Approach B".to_string()),
        GuestCommands::Compute("The QUICK brown Fox JUMPS over the LAZY dog".to_string()),
        GuestCommands::CallHost("Test nested bidirectional trait-based call".to_string()),
        GuestCommands::Custom {
            name: "filesystem_write".to_string(),
            data: "This is test data for the virtual filesystem".to_string(),
        },
        GuestCommands::Custom {
            name: "network_request".to_string(),
            data: "https://api.example.com/data".to_string(),
        },
    ];

    for cmd in commands {
        println!("\n[Host] Sending trait-based typed command: {:?}", cmd);
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
    println!("\nApproach B demonstrated:");
    println!("  ✓ Trait-based composable type-safe command system");
    println!("  ✓ Zero runtime serialization overhead");
    println!("  ✓ Host → Guest bidirectional communication");
    println!("  ✓ Guest → Host bidirectional communication");
    println!("  ✓ Nested calls (Guest calling Host during Host's request)");
    println!("  ✓ Compile-time type safety across WASM boundary");
    println!("  ✓ Composable interfaces (Filesystem + Network)");
    println!("  ✓ Interface extension via traits");

    Ok(())
}
