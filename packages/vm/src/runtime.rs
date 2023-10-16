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

#[derive(Clone)]
pub struct Runtime {
    engine: Engine,
    component: Component,
}

impl Runtime {
    pub fn new(bin: Bytes) -> Self {
        let mut config = Config::new();
        config.wasm_component_model(true);
        let engine = Engine::new(&config).unwrap();

        let component = unsafe { Component::deserialize(&engine, &bin).unwrap() };

        Self { engine, component }
    }

    pub fn run(&mut self) -> Result<()> {
        let mut linker = Linker::new(&self.engine);
        command::sync::add_to_linker(&mut linker).unwrap();

        let mut table = Table::new();
        let mut wasi = WasiCtxBuilder::new();
        wasi.inherit_stderr();
        wasi.stdin(InputStream::new(), wasmtime_wasi::preview2::IsATTY::No);
        wasi.stdout(OutputStream::new(), wasmtime_wasi::preview2::IsATTY::No);

        let wasi = wasi.build(&mut table).unwrap();

        let mut store = Store::new(&self.engine, Ctx { wasi, table });

        let (command, _) = Command::instantiate(&mut store, &self.component, &linker)?;

        command
            .wasi_cli_run()
            .call_run(&mut store)?
            .map_err(|()| anyhow!("guest command returned error"))?;

        Ok(())
    }
}
