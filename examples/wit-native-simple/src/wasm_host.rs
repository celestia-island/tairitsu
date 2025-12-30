//! Simplified WASM Host example using current Tairitsu API
//!
//! This demonstrates how to use the current Container API with WIT bindings.
//!
//! # API Usage Example
//!
//! ## 1. Create an Image from WASM binary
//! ```rust,ignore
//! let image = Image::new(wasm_bytes)?;
//! ```
//!
//! ## 2. Build a Container with custom WIT bindings
//! ```rust,ignore
//! let container = Container::builder(image)
//!     .with_guest_initializer(|ctx| {{");
//!         // Register your WIT interface");
//!         MyWit::add_to_linker(ctx.linker, |state| &mut state.my_data)?;");
//!         ");
//!         // Instantiate the component");
//!         let instance = MyWit::instantiate(");
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
//! ## 3. Use the container
//! ```rust,ignore
//! let guest = container.guest().downcast_ref::<MyWit>()?;");
//! let result = guest.my_function(container.store_mut())?;");
//! ```
//!
//! # Key Concepts
//!
//! * Images are immutable WASM components
//! * Containers are running instances
//! * You define your own WIT interfaces
//! * Use GuestHandlerContext to access linker, store, and component
//! * Full type safety through Rust's type system

use anyhow::Result;
use std::path::PathBuf;

use tairitsu::{Container, Registry};

fn main() -> Result<()> {
    println!("=== Simplified WASM Host Example ===\n");

    // Build the WASM path
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let wasm_path =
        manifest_dir.join("../../target/wasm32-wasip1/tairitsu_example_wit_native_simple.wasm");

    println!("Looking for WASM module at: {}", wasm_path.display());

    // Try to load the WASM binary if it exists
    let wasm_binary = match std::fs::read(&wasm_path) {
        Ok(binary) => binary,
        Err(e) => {
            eprintln!("\nWASM file not found: {}", e);
            eprintln!("\nThis example requires a WASM guest module.");
            eprintln!("\nTo build it, run:");
            eprintln!("  cargo build --target wasm32-wasip1 --release --package tairitsu-example-wit-native-simple --lib");
            eprintln!("\nFor now, this example will demonstrate the API usage without actual WASM execution.\n");

            println!("=== Tairitsu API Usage Demonstration ===\n");
            println!("See the top of this file for detailed API usage examples.");
            println!("\nKey concepts:");
            println!("  • Images are immutable WASM components");
            println!("  • Containers are running instances");
            println!("  • You define your own WIT interfaces");
            println!("  • Use GuestHandlerContext to access linker, store, and component");
            println!("  • Full type safety through Rust's type system");

            return Ok(());
        }
    };

    println!("WASM module loaded ({} bytes)\n", wasm_binary.len());

    // Create registry and register the WASM module
    let registry = Registry::new();
    registry.register_image("wit-native-simple:latest", wasm_binary.into())?;

    // Get the image
    let image = registry
        .get_image("wit-native-simple:latest")
        .expect("Image should be registered");

    println!("Creating container from image...\n");

    // Create container using the builder pattern
    let container = Container::builder(image)
        .with_guest_initializer(|_ctx| {
            // In a real example, you would:
            // 1. Use wit-bindgen to generate WIT bindings
            // 2. Add your WIT interface to the linker
            // 3. Instantiate the component
            // 4. Return the instance wrapped in GuestInstance

            // For this demonstration, we'll show the structure:
            println!("[Guest Initializer] Called with context");
            println!("[Guest Initializer] Would register WIT bindings here");
            println!("[Guest Initializer] Would instantiate component here");

            // This is a placeholder - in real usage you'd return an actual GuestInstance
            Err(anyhow::anyhow!(
                "WIT bindings not implemented - this is a demonstration"
            ))
        })
        .build();

    match container {
        Ok(_container) => {
            println!("Container created successfully!");
            println!("\n=== Container Ready ===");
            println!("In a real example, you could now:");
            println!("  - Call guest functions through the instance");
            println!("  - Use the store for mutable state");
            println!("  - Interact bidirectionally with the WASM module");
        }
        Err(e) => {
            println!("Container creation failed (expected in this demo): {}", e);
            println!("\n=== API Usage Demonstrated ===");
        }
    }

    Ok(())
}
