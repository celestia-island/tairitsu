//! Simplified WASM Host example for macro-based approach
//!
//! This demonstrates how to use the current Container API with macro-generated WIT interfaces.
//!
//! # API Usage Example
//!
//! ## 1. Define your WIT interface using macros
//! ```rust,ignore
//! wit_interface! {{");
//!     interface filesystem {{");
//!         read: func(path: String) -> Result<Vec<u8>, String>;");
//!         write: func(path: String, data: Vec<u8>) -> Result<(), String>;");
//!     }}");
//! }}");
//! ```
//!
//! ## 2. Create an Image from WASM binary
//! ```rust,ignore
//! let image = Image::new(wasm_bytes)?;
//! ```
//!
//! ## 3. Build a Container with macro-generated bindings
//! ```rust,ignore
//! let container = Container::builder(image)")
//!     .with_guest_initializer(|ctx| {{");
//!         // Register your macro-generated interface");
//!         Filesystem::add_to_linker(");
//!             ctx.linker,");
//!             |state| &mut state.filesystem");
//!         )?;");
//!         ");
//!         // Instantiate the component");
//!         let instance = Filesystem::instantiate(");
//!             ctx.store,");
//!             ctx.component,");
//!             ctx.linker");
//!         )?;");
//!         ");
//!         Ok(GuestInstance::new(instance))");
//!     }})?;");
//!     .build()?;");
//! ```
//!
//! # Key Advantages
//!
//! * Automatic WIT-to-Rust enum generation
//! * Zero boilerplate with macros
//! * Type-safe commands at compile time
//! * No runtime serialization overhead
//! * Easy composable interfaces

use anyhow::Result;
use log::info;
use std::path::PathBuf;

use tairitsu::{Container, Registry};

fn main() -> Result<()> {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    info!("=== Macro-Based WASM Host Example ===");

    // Build the WASM path
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let wasm_path = manifest_dir
        .join("../../target/wasm32-wasip2/release/tairitsu_example_wit_native_macro.wasm");

    info!("Looking for WASM module at: {}", wasm_path.display());

    // Try to load the WASM binary if it exists
    let wasm_binary = match std::fs::read(&wasm_path) {
        Ok(binary) => binary,
        Err(e) => {
            info!("WASM file not found: {}", e);
            info!("This example requires a WASM guest module.");
            info!("To build it, run:");
            info!("  cargo build --target wasm32-wasip2 --release --package tairitsu-example-wit-native-macro --lib");
            info!("For now, this example will demonstrate the API usage without actual WASM execution.");

            info!("See the top of this file for detailed API usage examples.");
            info!("Key advantages:");
            info!("  • Automatic WIT-to-Rust enum generation");
            info!("  • Zero boilerplate with macros");
            info!("  • Type-safe commands at compile time");
            info!("  • No runtime serialization overhead");
            info!("  • Easy composable interfaces");

            return Ok(());
        }
    };

    info!("WASM module loaded ({} bytes)", wasm_binary.len());

    // Create registry and register the WASM component
    // Use register_component() for wasip2 builds (Component Model)
    let registry = Registry::new();
    registry.register_component("wit-native-macro:latest", wasm_binary.into())?;

    // Get the image
    let image = registry
        .get_image("wit-native-macro:latest")
        .expect("Image should be registered");

    info!("Creating container from image...");

    // Create container using the builder pattern
    let container = Container::builder(image)
        .with_guest_initializer(|_ctx| {
            // In a real example, you would:
            // 1. Use wit-interface! macro to generate your WIT bindings
            // 2. Add your generated interface to the linker
            // 3. Instantiate the component
            // 4. Return the instance wrapped in GuestInstance

            // For this demonstration, we'll show the structure:
            info!("[Guest Initializer] Called with context");
            info!("[Guest Initializer] Would register macro-generated WIT bindings here");
            info!("[Guest Initializer] Would instantiate component here");

            // This is a placeholder - in real usage you'd return an actual GuestInstance
            Err(anyhow::anyhow!(
                "WIT bindings not implemented - this is a demonstration"
            ))
        })
        .build();

    match container {
        Ok(_container) => {
            info!("Container created successfully!");
            info!("=== Container Ready ===");
            info!("In a real example, you could now:");
            info!("  - Call guest functions through the instance");
            info!("  - Use macro-generated type-safe commands");
            info!("  - Interact bidirectionally with the WASM module");
        }
        Err(e) => {
            info!("Container creation failed (expected in this demo): {}", e);
            info!("=== API Usage Demonstrated ===");
        }
    }

    Ok(())
}
