use {
    anyhow::Result,
    bytes::Bytes,
    wasmtime::{Config, Engine},
    wit_component::ComponentEncoder,
};

mod runtime;
mod stream;

use runtime::generate_runtime;

fn main() -> Result<()> {
    // Transfer the wasm binary to wasm component binary

    println!("Downloading wasi adapter...");
    let adapter = &reqwest::blocking::get(
        "https://github.com/bytecodealliance/wasmtime/releases/download/\
         v13.0.0/wasi_snapshot_preview1.command.wasm",
    )?
    .error_for_status()?
    .bytes()?;

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
    generate_runtime(cwasm)?;

    Ok(())
}
