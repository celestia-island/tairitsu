// Test to understand wasmtime's handling of record types vs tuples in func_wrap
// This helps us understand the "expected 4-tuple, found 1-tuple" error

use wasmtime::component::Linker;
use wasmtime::{Store, Result};

#[derive(Clone)]
struct TestState {
    dummy: u32,
}

fn main() -> Result<()> {
    let engine = wasmtime::Engine::default();
    let mut linker = Linker::<TestState>::new(&engine);
    let state = TestState { dummy: 0 };
    let mut store = Store::new(&engine, state);

    // Test 1: Simple tuple return (should work)
    println!("Test 1: Simple tuple return");
    let mut instance = linker.instance("test:tuples@1.0.0")?;
    instance.func_wrap(
        "return-four-floats",
        |_caller: wasmtime::StoreContextMut<'_, TestState>,
         (): ()|
         -> Result<(f64, f64, f64, f64), wasmtime::Error> {
            Ok((1.0, 2.0, 3.0, 4.0))
        },
    )?;
    println!("✓ Simple tuple return registered successfully");

    // Test 2: Nested tuple return (what we currently have)
    println!("\nTest 2: Nested tuple return");
    instance.func_wrap(
        "return-nested-tuple",
        |_caller: wasmtime::StoreContextMut<'_, TestState>,
         (): ()|
         -> Result<((f64, f64, f64, f64),), wasmtime::Error> {
            Ok(((1.0, 2.0, 3.0, 4.0),))
        },
    )?;
    println!("✓ Nested tuple return registered successfully");

    // Test 3: Single value wrapped in tuple
    println!("\nTest 3: Single value wrapped in tuple");
    instance.func_wrap(
        "return-single-value",
        |_caller: wasmtime::StoreContextMut<'_, TestState>,
         (): ()|
         -> Result<(f64,), wasmtime::Error> {
            Ok((42.0,))
        },
    )?;
    println!("✓ Single value in tuple registered successfully");

    println!("\n✅ All func_wrap registrations succeeded!");
    println!("\nKey insight: wasmtime's func_wrap expects return types to match the Component Model's canonical ABI.");
    println!("For WIT record types like dom-rect, they should be returned as flattened tuples,");
    println!("not nested tuples. The error 'expected 4-tuple, found 1-tuple' suggests we're");
    println!("returning a nested tuple when wasmtime expects a flat tuple.");

    Ok(())
}