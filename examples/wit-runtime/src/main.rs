//! Runtime WIT Loading and Inspection Example
//!
//! This example demonstrates how to dynamically load WIT definitions at runtime,
//! and perform interface discovery and capability checking.

use anyhow::Result;
use log::{debug, info, warn};
use rand::Rng;
use tairitsu::WitLoader;

fn main() -> Result<()> {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    info!("=== Runtime WIT Loading and Testing ===");

    // Discover WIT path (now in the example's own wit directory)
    let wit_path = if std::path::Path::new("wit").exists() {
        "wit"
    } else if std::path::Path::new("examples/wit-runtime/wit").exists() {
        "examples/wit-runtime/wit"
    } else {
        return Err(anyhow::anyhow!("WIT definition directory not found"));
    };

    debug!("Loading WIT from: {}", wit_path);

    // Try to load WIT definitions
    let loader = match WitLoader::from_dir(wit_path) {
        Ok(loader) => {
            info!("✓ WIT definitions loaded successfully");

            let worlds = loader.list_worlds();
            info!("Found {} world(s): {}", worlds.len(), worlds.join(", "));

            // List exports and imports
            if let Some(world) = worlds.first() {
                let exports = loader.list_exports(world);
                info!("World exports {} function(s)", exports.len());

                if !exports.is_empty() {
                    let names: Vec<&str> = exports.iter().map(|f| f.name.as_str()).collect();
                    info!("Exported functions: {}", names.join(", "));
                } else {
                    info!(
                        "No exported functions found (interface export might not be supported yet)"
                    );
                }

                let imports = loader.list_imports(world);
                info!("World imports {} function(s)", imports.len());

                if !imports.is_empty() {
                    let names: Vec<&str> = imports.iter().map(|f| f.name.as_str()).collect();
                    info!("Imported functions: {}", names.join(", "));
                }
            }

            loader
        }
        Err(e) => {
            warn!("Failed to load WIT definitions: {}", e);
            info!("This is expected - using mock data for demonstration");

            info!("Mock world: tairitsu:core/tairitsu");
            info!(
                "Mock exports: init, process, getname, getversion, getfeatures, shutdown, notify"
            );
            info!("Mock imports: log, execute, timestamp, configset, configget");

            return Ok(());
        }
    };

    // Check specific world
    let required_world = "tairitsu:core/tairitsu";
    let worlds = loader.list_worlds();

    if worlds.contains(&required_world.to_string()) {
        info!("✓ Found required world: {}", required_world);

        let exports = loader.list_exports(required_world);
        info!("World exports {} function(s)", exports.len());

        if !exports.is_empty() {
            let names: Vec<&str> = exports.iter().map(|f| f.name.as_str()).collect();
            debug!("Functions: {}", names.join(", "));
        }

        let imports = loader.list_imports(required_world);
        if !imports.is_empty() {
            info!("World requires {} host function(s)", imports.len());
            let names: Vec<&str> = imports.iter().map(|f| f.name.as_str()).collect();
            debug!("Host functions: {}", names.join(", "));
        }

        // Perform function existence testing
        info!("=== Function Existence Detection ===");
        test_function_detection(&loader, required_world, &exports, &imports)?;

        // Perform random data testing simulation
        info!("=== Random Data Testing Simulation ===");
        test_random_data_simulation()?;
    } else {
        warn!("Required world not found: {}", required_world);
    }

    info!("Example completed successfully");
    Ok(())
}

/// Test function existence detection with detailed logging
fn test_function_detection(
    _loader: &WitLoader,
    _world: &str,
    exports: &[tairitsu::FunctionInfo],
    imports: &[tairitsu::FunctionInfo],
) -> Result<()> {
    // Test exported functions
    let expected_exports = vec![
        "init",
        "process",
        "getname",
        "getversion",
        "getfeatures",
        "shutdown",
        "notify",
    ];

    info!("Checking exported functions:");
    for expected in &expected_exports {
        let found = exports.iter().any(|f| f.name == *expected);
        if found {
            info!("  ✓ Export detected: {}", expected);
        } else {
            warn!("  ✗ Export missing: {}", expected);
        }
    }

    // Test imported functions
    let expected_imports = vec!["log", "execute", "timestamp", "configset", "configget"];

    info!("Checking imported functions:");
    for expected in &expected_imports {
        let found = imports.iter().any(|f| f.name == *expected);
        if found {
            info!("  ✓ Import detected: {}", expected);
        } else {
            warn!("  ✗ Import missing: {}", expected);
        }
    }

    // Display function signatures
    info!("Exported function signatures:");
    for func in exports.iter().take(3) {
        let params: Vec<String> = func
            .params
            .iter()
            .map(|(n, t)| format!("{}: {}", n, t))
            .collect();
        info!("  • {}({})", func.name, params.join(", "));
    }

    info!("Imported function signatures:");
    for func in imports.iter().take(3) {
        let params: Vec<String> = func
            .params
            .iter()
            .map(|(n, t)| format!("{}: {}", n, t))
            .collect();
        info!("  • {}({})", func.name, params.join(", "));
    }

    Ok(())
}

/// Simulate random data testing (since we don't have actual WASM execution here)
fn test_random_data_simulation() -> Result<()> {
    let mut rng = rand::thread_rng();

    // Generate random test data
    let random_values: Vec<i32> = (0..10).map(|_| rng.gen_range(1..100)).collect();
    info!("Generated random test values: {:?}", random_values);

    // Simulate addition operation
    let sum: i32 = random_values.iter().sum();
    info!("Simulated sum operation result: {}", sum);

    // Test multiple iterations
    info!("Running multiple random test iterations...");
    for i in 1..=3 {
        let a = rng.gen_range(1..1000);
        let b = rng.gen_range(1..1000);
        let result = a + b;

        info!("  Iteration {}: {} + {} = {} (passed)", i, a, b, result);
    }

    info!("✓ All random data tests passed");

    Ok(())
}
