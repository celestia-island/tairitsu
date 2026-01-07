//! Dynamic WASM Component Invocation Example
//!
//! This example demonstrates the new dynamic invocation features in Tairitsu 0.3.0:
//! - Runtime dynamic calling of guest exports using RON serialization
//! - Binary canonical ABI calling for high performance
//! - Host import registration and invocation
//! - Runtime discovery of available functions

use anyhow::Result;
use log::{debug, error, info, warn};

use tairitsu::{
    dynamic::host_imports::HostImportRegistry,
    ron::{typed_ron_tool, RonBinding, RonToolRegistry},
    Container, Image,
};

// ============================================================================
// Host Import Implementation
// ============================================================================

#[allow(dead_code)]
struct LoggerImport;

#[allow(dead_code)]
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
    info!("║  Tairitsu 0.3.0 Dynamic Invocation Example                ║");
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

    info!(
        "Registered host imports: {:?}",
        host_registry.list_imports()
    );

    // Test host import call
    match host_registry.call(
        "host-add",
        &[
            wasmtime::component::Val::S32(10),
            wasmtime::component::Val::S32(32),
        ],
    ) {
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
    // ========================================================================
    // Scenario 6: Complex Type Support Verification
    // ========================================================================
    info!("\n🧪 Scenario 6: Complex Type Support Verification");

    use tairitsu::dynamic::{ron_to_val, val_to_ron};
    use wasmtime::component::Val;

    // Test 1: List type
    info!("\n  [Test 1] List type (Vec<u32>)");
    let list_val = Val::List(vec![Val::U32(1), Val::U32(2), Val::U32(3)]);
    let list_ron = val_to_ron(&list_val)?;
    info!("    Serialized: {}", list_ron);
    assert_eq!(list_ron, "[1, 2, 3]");

    // Test 2: Tuple type
    info!("\n  [Test 2] Tuple type (String, u32)");
    let tuple_val = Val::Tuple(vec![Val::String("hello".to_string()), Val::U32(42)]);
    let tuple_ron = val_to_ron(&tuple_val)?;
    info!("    Serialized: {}", tuple_ron);
    assert_eq!(tuple_ron, "(\"hello\", 42)");

    // Test 3: Option type - Some
    info!("\n  [Test 3] Option type - Some(u32)");
    let some_val = Val::Option(Some(Box::new(Val::U32(100))));
    let some_ron = val_to_ron(&some_val)?;
    info!("    Serialized: {}", some_ron);
    assert_eq!(some_ron, "Some(100)");

    // Test 4: Option type - None
    info!("\n  [Test 4] Option type - None");
    let none_val = Val::Option(None);
    let none_ron = val_to_ron(&none_val)?;
    info!("    Serialized: {}", none_ron);
    assert_eq!(none_ron, "None");

    // Test 5: Result type - Ok
    info!("\n  [Test 5] Result type - Ok(u32)");
    let ok_val = Val::Result(Ok(Some(Box::new(Val::U32(200)))));
    let ok_ron = val_to_ron(&ok_val)?;
    info!("    Serialized: {}", ok_ron);
    assert_eq!(ok_ron, "Ok(200)");

    // Test 6: Result type - Err
    info!("\n  [Test 6] Result type - Err(string)");
    let err_val = Val::Result(Err(Some(Box::new(Val::String("error".to_string())))));
    let err_ron = val_to_ron(&err_val)?;
    info!("    Serialized: {}", err_ron);
    assert_eq!(err_ron, "Err(\"error\")");

    // Test 7: Result type - Ok with unit
    info!("\n  [Test 7] Result type - Ok with unit");
    let ok_unit_val = Val::Result(Ok(None));
    let ok_unit_ron = val_to_ron(&ok_unit_val)?;
    info!("    Serialized: {}", ok_unit_ron);
    assert_eq!(ok_unit_ron, "Ok(())");

    // Test 8: Record type
    info!("\n  [Test 8] Record type");
    let record_val = Val::Record(vec![
        ("name".to_string(), Val::String("test".to_string())),
        ("value".to_string(), Val::U32(999)),
    ]);
    let record_ron = val_to_ron(&record_val)?;
    info!("    Serialized: {}", record_ron);
    assert!(record_ron.contains("name"));
    assert!(record_ron.contains("test"));
    assert!(record_ron.contains("value"));
    assert!(record_ron.contains("999"));

    // Test 9: Float32 type
    info!("\n  [Test 9] Float32 type");
    let f32_val = Val::Float32(std::f32::consts::FRAC_PI_4);
    let f32_ron = val_to_ron(&f32_val)?;
    info!("    Serialized: {}", f32_ron);
    assert!(f32_ron.contains("e")); // Scientific notation

    // Test 10: Float64 type
    info!("\n  [Test 10] Float64 type");
    let f64_val = Val::Float64(std::f64::consts::LN_2);
    let f64_ron = val_to_ron(&f64_val)?;
    info!("    Serialized: {}", f64_ron);
    assert!(f64_ron.contains("e")); // Scientific notation

    info!("\n  ✅ All complex type serialization tests passed!");

    // ========================================================================
    // Scenario 7: RON Deserialization Tests
    // ========================================================================
    info!("\n🔄 Scenario 7: RON Deserialization");

    use wasmtime::component::Type;

    // Test basic type deserialization
    info!("\n  [Test 1] Basic type - Bool");
    let bool_result = ron_to_val("true", &Type::Bool)?;
    assert!(matches!(bool_result, Val::Bool(true)));
    info!("    ✓ Deserialized true");

    info!("\n  [Test 2] Basic type - U32");
    let u32_result = ron_to_val("42", &Type::U32)?;
    assert!(matches!(u32_result, Val::U32(42)));
    info!("    ✓ Deserialized 42");

    info!("\n  [Test 3] Basic type - String");
    let str_result = ron_to_val("\"hello\"", &Type::String)?;
    assert!(matches!(str_result, Val::String(_)));
    info!("    ✓ Deserialized string");

    info!("\n  [Test 4] Float type - Float32");
    let f32_result = ron_to_val("1.5", &Type::Float32)?;
    assert!(matches!(f32_result, Val::Float32(_)));
    info!("    ✓ Deserialized Float32");

    info!("\n  [Test 5] Float type - Float64");
    let f64_result = ron_to_val("1.0", &Type::Float64)?;
    assert!(matches!(f64_result, Val::Float64(_)));
    info!("    ✓ Deserialized Float64");

    info!("\n  ✅ All deserialization tests passed!");

    // ========================================================================
    // Scenario 8: API Usage Patterns
    // ========================================================================

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
    // Scenario 9: Error Handling
    // ========================================================================
    info!("\n⚠️  Scenario 9: Error Handling");

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
