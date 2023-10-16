use {
    anyhow::Result,
    bytes::Bytes,
    wasmtime::{Config, Engine},
    wit_component::ComponentEncoder,
};

mod runtime;
mod stream;

use runtime::Runtime;

fn main() -> Result<()> {
    // Transfer the wasm binary to wasm component binary

    let adapter = include_bytes!("../res/wasi_snapshot_preview1.command.wasm");

    let component = &ComponentEncoder::default()
        .module(include_bytes!(
            "../../../target/wasm32-wasi/release/tairitsu-proto.wasm"
        ))?
        .validate(true)
        .adapter("wasi_snapshot_preview1", adapter)?
        .encode()?;

    let mut config = Config::new();
    config.wasm_component_model(true);

    let engine = &Engine::new(&config)?;

    let cwasm = engine.precompile_component(component)?;
    let cwasm = Bytes::from(cwasm);

    // Run the prototype demo

    println!("Running prototype demo...");
    let entity_base = Runtime::new(cwasm);

    use std::time::Instant;

    let begin = Instant::now();
    for _ in 0..1000 {
        entity_base.clone().run()?;
    }
    let end = Instant::now();
    println!("Time elapsed: {:?}", end - begin);

    Ok(())
}
