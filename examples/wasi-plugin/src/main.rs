use anyhow::Result;
use std::path::PathBuf;

use bytes::Bytes;
use tairitsu::{Container, ContainerState, GuestInstance, Image};

fn main() -> Result<()> {
    let wasm_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/wasm32-wasip2/release/tairitsu_example_wasi_plugin.wasm");

    let wasm_binary = match std::fs::read(&wasm_path) {
        Ok(b) => b,
        Err(_) => {
            eprintln!("WASM not found at {}. Build with:", wasm_path.display());
            eprintln!("  cargo build --target wasm32-wasip2 --release -p tairitsu-example-wasi-plugin --lib");
            std::process::exit(1);
        }
    };
    eprintln!("Loaded WASM component ({} bytes)", wasm_binary.len());

    smoke_basic(&wasm_binary)?;
    smoke_fuel(&wasm_binary)?;
    smoke_state(&wasm_binary)?;
    smoke_multi_container(&wasm_binary)?;

    eprintln!("\nAll smoke tests passed.");
    Ok(())
}

fn make_initializer(
) -> impl for<'a> FnOnce(tairitsu::GuestHandlerContext<'a, tairitsu::HostState>) -> Result<GuestInstance>
{
    |ctx| {
        let instance = ctx
            .linker
            .instantiate(ctx.store, ctx.component)
            .map_err(|e| anyhow::anyhow!("instantiation failed: {}", e))?;
        Ok(GuestInstance::new_dynamic(instance))
    }
}

fn smoke_basic(wasm: &[u8]) -> Result<()> {
    eprintln!("\n=== basic: Image::from_component + dynamic call ===");

    let image = Image::from_component(Bytes::copy_from_slice(wasm))?;
    let mut container = Container::builder(image)
        .with_guest_initializer(make_initializer())
        .build()?;

    let result = container.call_guest_raw_desc("greet", r#""world""#)?;
    eprintln!("  greet(\"world\") = {}", result);

    let result = container.call_guest_raw_desc("add", r#"(3, 4)"#)?;
    eprintln!("  add(3, 4) = {}", result);
    assert!(result.contains('7'), "add(3,4) should be 7");

    eprintln!("  basic: OK");
    Ok(())
}

fn smoke_fuel(wasm: &[u8]) -> Result<()> {
    eprintln!("\n=== fuel: Image::from_component_with_config + with_fuel_limit ===");

    let mut config = tairitsu::Config::new();
    config.wasm_component_model(true).consume_fuel(true);

    let image = Image::from_component_with_config(Bytes::copy_from_slice(wasm), config)?;
    let mut container = Container::builder(image)
        .with_fuel_limit(10_000)
        .with_guest_initializer(make_initializer())
        .build()?;

    let fuel_before = container.store().get_fuel()?;
    eprintln!("  fuel before call: {}", fuel_before);

    let result = container.call_guest_raw_desc("add", r#"(1, 1)"#)?;
    eprintln!("  add(1, 1) = {}", result);

    let fuel_after = container.store().get_fuel()?;
    eprintln!("  fuel after call:  {}", fuel_after);
    assert!(
        fuel_after < fuel_before,
        "fuel should decrease after a call"
    );

    eprintln!("  fuel: OK");
    Ok(())
}

fn smoke_state(wasm: &[u8]) -> Result<()> {
    eprintln!("\n=== state: ContainerState transitions ===");

    let image = Image::from_component(Bytes::copy_from_slice(wasm))?;
    let mut container = Container::builder(image)
        .with_guest_initializer(make_initializer())
        .build()?;

    assert_eq!(container.state(), &ContainerState::Created);
    eprintln!("  initial state: Created");

    let _ = container.call_guest_raw_desc("add", r#"(0, 0)"#)?;
    assert_eq!(container.state(), &ContainerState::Running);
    eprintln!("  after call:    Running");

    container.stop();
    assert_eq!(container.state(), &ContainerState::Stopped);
    eprintln!("  after stop:    Stopped");

    let err = container.call_guest_raw_desc("add", r#"(1, 2)"#);
    assert!(err.is_err(), "calling on stopped container should fail");
    eprintln!("  call after stop: Err (expected)");

    eprintln!("  state: OK");
    Ok(())
}

fn smoke_multi_container(wasm: &[u8]) -> Result<()> {
    eprintln!("\n=== multi-container: two containers from same Image ===");

    let image = Image::from_component(Bytes::copy_from_slice(wasm))?;

    let make_container = |img: Image| -> Result<_> {
        Container::builder(img)
            .with_guest_initializer(make_initializer())
            .build()
    };

    let mut c1 = make_container(image.clone())?;
    let mut c2 = make_container(image)?;

    let r1 = c1.call_guest_raw_desc("greet", r#""container-1""#)?;
    let r2 = c2.call_guest_raw_desc("greet", r#""container-2""#)?;

    eprintln!("  c1: {}", r1);
    eprintln!("  c2: {}", r2);

    eprintln!("  multi-container: OK");
    Ok(())
}
