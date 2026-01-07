//! Dynamic WASM Component Invocation Example
//!
//! This example demonstrates the new dynamic invocation features in Tairitsu 0.2.2:
//! - Runtime dynamic calling of guest exports using RON serialization
//! - Binary canonical ABI calling for high performance
//! - Host import registration and invocation
//! - Runtime discovery of available functions

use anyhow::Result;
use log::{debug, error, info, warn};

use wasmtime::component::Type;

use tairitsu::{Container, HostState, Image, dynamic::host_imports::HostImportRegistry, ron::{RonBinding, RonToolRegistry, typed_ron_tool}};

// ============================================================================
// Host Import Implementation
// ============================================================================

struct LoggerImport;

impl LoggerImport {
    fn log(level: String, message: String) -> String {
        println!("[{}] {}", level, message);
        format!("Logged: {}", message)
    }
}

// ============================================================================
// Example Tool Implementations for Comparison
// ============================================================================

#[derive(serde::Deserialize)]
struct ReverseInput {
    text: String,
}

#[derive(serde::Serialize)]
struct ReverseOutput {
    result: String,
}

// ============================================================================
// Main Demo
// ============================================================================

fn main() -> Result<()> {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    info!("╔════════════════════════════════════════════════════════════╗");
    info!("║  Tairitsu 0.2.2 Dynamic Invocation Example                ║");
    info!("╚════════════════════════════════════════════════════════════╝");

    // ========================================================================
    // Scenario 1: Load WASM Component
    // ========================================================================
    info!("\n📦 Scenario 1: Loading WASM Component");

    // In a real scenario, you would load an actual WASM component file
    // For this demo, we'll show how the API would work
    info!("Note: This example demonstrates the API design.");
    info!("In production, you would:");
    info!("  1. Compile guest WASM component from WIT definitions");
    info!("  2. Load the component binary");
    info!("  3. Create a Container with dynamic support");

    // ========================================================================
    // Scenario 2: RON Serialization Demo
    // ========================================================================
    info!("\n🔤 Scenario 2: RON Serialization");

    #[derive(serde::Serialize, serde::Deserialize, Debug)]
    struct CalculatorRequest {
        a: i32,
        b: i32,
    }

    let request = CalculatorRequest { a: 42, b: 58 };
    let ron = RonBinding::params_to_ron(&request)?;
    info!("RON serialized: {}", ron);

    let decoded: CalculatorRequest = RonBinding::ron_to_params(&ron)?;
    info!("RON deserialized: {:?}", decoded);

    // ========================================================================
    // Scenario 3: RonTool Registry (Comparison with ToolRegistry)
    // ========================================================================
    info!("\n🔧 Scenario 3: RonTool Registry");

    let reverse_tool = typed_ron_tool("reverse", |input: ReverseInput| -> ReverseOutput {
        ReverseOutput {
            result: input.text.chars().rev().collect(),
        }
    });

    let mut ron_registry = RonToolRegistry::new();
    ron_registry.register("reverse".to_string(), reverse_tool);

    info!("Registered RON tools: {:?}", ron_registry.list_tools());

    let result = ron_registry.invoke("reverse", r#"(text: "Hello, World!")"#)?;
    info!("RON tool result: {}", result);

    // ========================================================================
    // Scenario 4: Host Import Registration
    // ========================================================================
    info!("\n📥 Scenario 4: Host Import Registration");

    let mut host_registry = HostImportRegistry::new();

    // Register a simple logging function
    host_registry.register(
        "host-log".to_string(),
        vec![Type::String, Type::String], // level, message
        vec![Type::String],               // return message
        |args| {
            use wasmtime::component::Val;
            let level = match &args[0] {
                Val::String(s) => s.as_str(),
                _ => return Err(anyhow::anyhow!("Invalid level type")),
            };
            let message = match &args[1] {
                Val::String(s) => s.as_str(),
                _ => return Err(anyhow::anyhow!("Invalid message type")),
            };

            println!("[HOST LOG] [{}] {}", level, message);
            Ok(vec![Val::String(format!("Logged: {}", message))])
        },
    );

    // Register a calculator function
    host_registry.register(
        "host-add".to_string(),
        vec![Type::S32, Type::S32],
        vec![Type::S32],
        |args| {
            use wasmtime::component::Val;
            let a = match &args[0] {
                Val::S32(n) => *n,
                _ => return Err(anyhow::anyhow!("Invalid type")),
            };
            let b = match &args[1] {
                Val::S32(n) => *n,
                _ => return Err(anyhow::anyhow!("Invalid type")),
            };
            Ok(vec![Val::S32(a + b)])
        },
    );

    info!("Registered host imports: {:?}", host_registry.list_imports());

    // Test host import call
    match host_registry.call("host-add", &[wasmtime::component::Val::S32(10), wasmtime::component::Val::S32(32)]) {
        Ok(results) => {
            if let Some(wasmtime::component::Val::S32(result)) = results.first() {
                info!("host-add result: 10 + 32 = {}", result);
            }
        }
        Err(e) => error!("host-add failed: {}", e),
    }

    // ========================================================================
    // Scenario 5: Runtime Discovery API
    // ========================================================================
    info!("\n🔍 Scenario 5: Runtime Discovery");

    let imports = host_registry.list_imports();
    info!("Available host imports:");
    for import_name in &imports {
        if let Some((params, results)) = host_registry.get_signature(import_name) {
            info!("  - {}: {:?} -> {:?}", import_name, params, results);
        }
    }

    // ========================================================================
    // Scenario 6: API Usage Patterns (Pseudo-code Examples)
    // ========================================================================
    info!("\n📚 Scenario 6: API Usage Patterns");

    info!("\n--- Pattern 1: Guest Export with RON ---");
    info!("let result = container.call_guest_raw_desc(");
    info!("    \"add\",");
    info!(r#"    "(a: 42, b: 58)")"#);
    info!(")?;");
    info!("// Returns: \"100\"");

    info!("\n--- Pattern 2: Guest Export with Binary ---");
    info!("use wasmtime::component::Val;");
    info!("let args = vec![Val::S32(42), Val::S32(58)];");
    info!("let results = container.call_guest_binary(\"add\", &args)?;");
    info!("// Returns: vec![Val::S32(100)]");

    info!("\n--- Pattern 3: Host Import with RON ---");
    info!("container.register_host_import(...);");
    info!("let result = container.call_host_import_raw_desc(");
    info!("    \"host-log\",");
    info!(r#"    "[\"info\", \"Hello from guest!\"]""#);
    info!(")?;");

    info!("\n--- Pattern 4: Function Discovery ---");
    info!("let exports = container.list_guest_exports()?;");
    info!("for export in exports {{");
    info!("    println!(\"Function: {{}}\", export.name);");
    info!("}}");

    // ========================================================================
    // Scenario 7: Error Handling
    // ========================================================================
    info!("\n⚠️  Scenario 7: Error Handling");

    // Test calling non-existent tool
    match ron_registry.invoke("non-existent", "test") {
        Ok(_) => warn!("Non-existent tool unexpectedly succeeded"),
        Err(e) => debug!("Expected error: {}", e),
    }

    // Test invalid RON
    match ron_registry.invoke("reverse", "invalid ron") {
        Ok(_) => warn!("Invalid RON unexpectedly succeeded"),
        Err(e) => debug!("Expected error: {}", e),
    }

    // ========================================================================
    // Summary
    // ========================================================================
    info!("\n✅ Summary: Dynamic Invocation API Features");
    info!("  ✓ RON serialization for Rust-friendly types");
    info!("  ✓ Binary canonical ABI for performance");
    info!("  ✓ Host import registration and invocation");
    info!("  ✓ Runtime function discovery");
    info!("  ✓ Backward compatible with existing APIs");

    info!("\n📖 Next Steps:");
    info!("  1. Compile guest WASM component from WIT definitions");
    info!("  2. Use Container::builder() with dynamic feature");
    info!("  3. Call guest exports dynamically with call_guest_raw_desc()");
    info!("  4. Register host imports with HostImportRegistry");

    info!("\n🎉 Example completed successfully!");
    Ok(())
}

// ============================================================================
// Helper: Example Container Creation (Pseudo-code)
// ============================================================================

#[allow(dead_code)]
fn example_container_creation() -> Result<()> {
    use bytes::Bytes;

    // Step 1: Load WASM binary
    let wasm_binary = std::fs::read("guest_component.wasm")?;
    let image = Image::new(Bytes::from(wasm_binary))?;

    // Step 2: Build container with host imports
    let builder = Container::builder(image);
    // In real usage, you would call:
    // let mut container = builder
    //     .with_host_state(HostState::new()?)
    //     .with_guest_initializer(|ctx| {
    //         // Register WIT interface
    //         // MyWit::add_to_linker(ctx.linker, |state| &mut state.my_data)?;
    //         // let instance = MyWit::instantiate(ctx.store, ctx.component, ctx.linker)?;
    //         // Ok(GuestInstance::new(instance))
    //         Ok(GuestInstance::new(()))
    //     })?
    //     .build()?;

    let _ = builder;
    info!("Note: Container creation requires actual WASM component with WIT bindings");
    Ok(())
}
