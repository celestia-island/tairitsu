//! WASM Host example with WIT bindings
//!
//! This demonstrates how to use Tairitsu with WIT bindings for
//! bidirectional host-guest communication using the Component Model.
//!
//! # Key Concepts
//!
//! * Images are immutable WASM components
//! * Containers are running instances
//! * WIT bindings are generated from .wit files
//! * Host and guest communicate through typed interfaces
//! * Full type safety through Rust's type system
//!
//! # Architecture
//!
//! ## 1. Host provides functions to guest (import)
//! The host implements the `host-api` interface that the guest can call:
//! - `log(message)` - Log messages from guest
//! - `execute_command(command, args)` - Execute commands
//!
//! ## 2. Guest provides functions to host (export)
//! The guest implements the `guest-api` interface that the host can call:
//! - `init()` - Initialize the guest module
//! - `process(input)` - Process requests
//! - `get_info()` - Get guest information

use anyhow::Result;
use log::info;
use std::path::PathBuf;

use tairitsu::{Container, Registry};

fn main() -> Result<()> {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    info!("=== WASM Host with WIT Bindings Example ===");

    // Build the WASM path
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let wasm_path = manifest_dir
        .join("../../target/wasm32-wasip2/release/tairitsu_example_wit_native_simple.wasm");

    info!("Looking for WASM component at: {}", wasm_path.display());

    // Try to load the WASM binary
    let wasm_binary = match std::fs::read(&wasm_path) {
        Ok(binary) => binary,
        Err(e) => {
            info!("WASM file not found: {}", e);
            info!("This example requires a WASM guest module.");
            info!("To build it, run:");
            info!("  cargo build --target wasm32-wasip2 --release --package tairitsu-example-wit-native-simple --lib");
            info!("For now, showing API usage demonstration.");

            demonstrate_api_usage();
            return Ok(());
        }
    };

    info!("WASM component loaded ({} bytes)", wasm_binary.len());

    // Create registry and register the WASM component
    // Use register_component() for wasip2 builds (Component Model)
    let registry = Registry::new();
    registry.register_component("wit-simple:latest", wasm_binary.into())?;

    // Get the image
    let image = registry
        .get_image("wit-simple:latest")
        .expect("Image should be registered");

    info!("Creating container from image...");

    // Create container using the builder pattern with WIT bindings
    let container = Container::builder(image)
        .with_guest_initializer(|_ctx| {
            // In a complete WIT implementation, you would:
            //
            // 1. Add the host's host-api interface to the linker
            //    HostImpl::add_to_linker(ctx.linker, |state| &mut state.host_data)?;
            //
            // 2. Instantiate the guest's guest-api interface
            //    let guest = Guest::instantiate(ctx.store, ctx.component, ctx.linker)?;
            //
            // 3. Store the instance for later use
            //    Ok(GuestInstance::new(guest))

            info!("[Guest Initializer] Setting up WIT bindings...");
            info!("[Guest Initializer] Host API registered");
            info!("[Guest Initializer] Guest API instantiated");

            // This is a demonstration - actual WIT bindings would be generated
            Err(anyhow::anyhow!(
                "Complete WIT bindings implementation pending - see documentation"
            ))
        })
        .build();

    match container {
        Ok(_container) => {
            info!("Container created successfully!");
            info!("=== WIT Bidirectional Communication ===");
            info!("In a complete implementation, you could:");
            info!("  • Host calls guest: guest.process(input)");
            info!("  • Guest calls host: host.log(message)");
            info!("  • Full type safety through WIT-generated interfaces");
        }
        Err(e) => {
            info!("Container creation (expected in this demo): {}", e);
        }
    }

    Ok(())
}

fn demonstrate_api_usage() {
    info!("=== Tairitsu WIT API Usage ===");
    info!("1. Define WIT interfaces (../../wit/tairitsu.wit):");
    info!("   • host-api: Functions host provides to guest");
    info!("   • guest-api: Functions guest provides to host");
    info!("2. Generate bindings with wit_bindgen::generate!()");
    info!("   • Host generates imports (guest calls host)");
    info!("   • Guest generates exports (host calls guest)");
    info!("3. Use Container builder with WIT:");
    info!("   let container = Container::builder(image)");
    info!("       .with_guest_initializer(|ctx| {{");
    info!("         // Add host API to linker");
    info!("         HostImpl::add_to_linker(ctx.linker, ...)?;");
    info!("         // Instantiate guest API");
    info!("         let guest = Guest::instantiate(ctx.store, ...)?;");
    info!("         Ok(GuestInstance::new(guest))");
    info!("       }})?;");
    info!("       .build()?;");
    info!("4. Bidirectional communication:");
    info!("   guest.process(input)  // Host → Guest");
    info!("   host.log(message)    // Guest → Host");
}
