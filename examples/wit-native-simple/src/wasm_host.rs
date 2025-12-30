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
use std::path::PathBuf;

use tairitsu::{Container, Registry};

fn main() -> Result<()> {
    println!("=== WASM Host with WIT Bindings Example ===\n");

    // Build the WASM path
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let wasm_path =
        manifest_dir.join("../../target/wasm32-wasip1/tairitsu_example_wit_native_simple.wasm");

    println!("Looking for WASM component at: {}", wasm_path.display());

    // Try to load the WASM binary
    let wasm_binary = match std::fs::read(&wasm_path) {
        Ok(binary) => binary,
        Err(e) => {
            eprintln!("\nWASM file not found: {}", e);
            eprintln!("\nThis example requires a WASM guest module.");
            eprintln!("\nTo build it, run:");
            eprintln!("  just build-simple-wasm");
            eprintln!("\nFor now, showing API usage demonstration.\n");

            demonstrate_api_usage();
            return Ok(());
        }
    };

    println!("WASM component loaded ({} bytes)\n", wasm_binary.len());

    // Create registry and register the WASM module
    let registry = Registry::new();
    registry.register_image("wit-simple:latest", wasm_binary.into())?;

    // Get the image
    let image = registry
        .get_image("wit-simple:latest")
        .expect("Image should be registered");

    println!("Creating container from image...\n");

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

            println!("[Guest Initializer] Setting up WIT bindings...");
            println!("[Guest Initializer] Host API registered");
            println!("[Guest Initializer] Guest API instantiated");

            // This is a demonstration - actual WIT bindings would be generated
            Err(anyhow::anyhow!(
                "Complete WIT bindings implementation pending - see documentation"
            ))
        })
        .build();

    match container {
        Ok(_container) => {
            println!("Container created successfully!");
            println!("\n=== WIT Bidirectional Communication ===");
            println!("In a complete implementation, you could:");
            println!("  • Host calls guest: guest.process(input)");
            println!("  • Guest calls host: host.log(message)");
            println!("  • Full type safety through WIT-generated interfaces");
        }
        Err(e) => {
            println!("Container creation (expected in this demo): {}", e);
        }
    }

    Ok(())
}

fn demonstrate_api_usage() {
    println!("=== Tairitsu WIT API Usage ===\n");

    println!("1. Define WIT interfaces (../../wit/tairitsu.wit):");
    println!("   • host-api: Functions host provides to guest");
    println!("   • guest-api: Functions guest provides to host");
    println!();

    println!("2. Generate bindings with wit_bindgen::generate!()");
    println!("   • Host generates imports (guest calls host)");
    println!("   • Guest generates exports (host calls guest)");
    println!();

    println!("3. Use Container builder with WIT:");
    println!("   let container = Container::builder(image)");
    println!("       .with_guest_initializer(|ctx| {{");
    println!("         // Add host API to linker");
    println!("         HostImpl::add_to_linker(ctx.linker, ...)?;");
    println!();
    println!("         // Instantiate guest API");
    println!("         let guest = Guest::instantiate(ctx.store, ...)?;");
    println!();
    println!("         Ok(GuestInstance::new(guest))");
    println!("       }})?;");
    println!("       .build()?;");
    println!();

    println!("4. Bidirectional communication:");
    println!("   guest.process(input)  // Host → Guest");
    println!("   host.log(message)    // Guest → Host");
}
