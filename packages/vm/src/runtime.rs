use {
    anyhow::{anyhow, Result},
    bytes::Bytes,
    wasmtime::{
        component::{Component, Linker},
        Config, Engine, Store,
    },
    wasmtime_wasi::preview2::{
        command::{self, sync::Command},
        Table, WasiCtx, WasiCtxBuilder, WasiView,
    },
};

use crate::stream::{InputStream, OutputStream};

struct Ctx {
    wasi: WasiCtx,
    table: Table,
}

impl WasiView for Ctx {
    fn ctx(&self) -> &WasiCtx {
        &self.wasi
    }
    fn ctx_mut(&mut self) -> &mut WasiCtx {
        &mut self.wasi
    }
    fn table(&self) -> &Table {
        &self.table
    }
    fn table_mut(&mut self) -> &mut Table {
        &mut self.table
    }
}

// TODO - Write a dynamic version of this function
pub fn generate_runtime(bin: Bytes) -> Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);

    use std::time::Instant;
    let start_time = Instant::now();

    for _ in 0..1000 {
        let engine = &Engine::new(&config)?;

        let mut linker = Linker::new(engine);
        command::sync::add_to_linker(&mut linker)?;

        let mut table = Table::new();
        let mut wasi = WasiCtxBuilder::new();
        wasi.inherit_stderr();
        wasi.stdin(InputStream::new(), wasmtime_wasi::preview2::IsATTY::No);
        wasi.stdout(OutputStream::new(), wasmtime_wasi::preview2::IsATTY::No);

        let wasi = wasi.build(&mut table)?;

        let mut store = Store::new(engine, Ctx { wasi, table });

        let (command, _) = Command::instantiate(
            &mut store,
            &unsafe { Component::deserialize(engine, &bin) }?,
            &linker,
        )?;

        command
            .wasi_cli_run()
            .call_run(&mut store)?
            .map_err(|()| anyhow!("guest command returned error"))?;
    }

    let end_time = Instant::now();
    let elapsed_time = end_time.duration_since(start_time);
    println!("Elapsed time {}ms on 1000 round", elapsed_time.as_millis());

    Ok(())
}
