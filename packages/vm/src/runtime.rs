use anyhow::{anyhow, Result};
use bytes::Bytes;
use flume::{Receiver, Sender};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

use wasmtime::{
    component::{Component, Linker},
    Config, Engine, Store,
};
use wasmtime_wasi::preview2::{
    command::{self, sync::Command},
    Table, WasiCtx, WasiCtxBuilder, WasiView,
};
use wit_component::ComponentEncoder;

use crate::stream::{HostInputStreamBox, HostOutputStreamBox};

lazy_static! {
    static ref ADAPTER: Bytes =
        Bytes::from_static(include_bytes!("../res/wasi_snapshot_preview1.command.wasm"));
}

pub struct WasiContext {
    wasi: WasiCtx,
    table: Table,
}

impl WasiView for WasiContext {
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
pub struct Image {
    pub engine: Engine,
    pub component: Component,
}

impl std::fmt::Debug for Image {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("<Runtime>").finish()
    }
}

#[derive(Clone)]
pub struct Container<Res: Serialize, Req: Serialize> {
    pub store: Arc<Mutex<Store<WasiContext>>>,
    pub component: Component,
    pub linker: Linker<WasiContext>,

    pub tx: Sender<Res>,
    pub rx: Receiver<Req>,
}

impl<Res: Serialize, Req: Serialize> std::fmt::Debug for Container<Res, Req> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("<Runner>").finish()
    }
}

impl Image {
    pub fn new(bin: Bytes) -> Self {
        let mut config = Config::new();
        config.wasm_component_model(true);
        let engine = Engine::new(&config).expect("Cannot create engine");

        // Transfer the wasm binary to wasm component binary
        let component = ComponentEncoder::default()
            .module(bin.as_ref())
            .expect("Cannot parse module binary")
            .validate(true)
            .adapter("wasi_snapshot_preview1", ADAPTER.as_ref())
            .expect("Cannot find adapter")
            .encode()
            .expect("Cannot encode the wasm component");

        let cwasm = Bytes::from(
            engine
                .precompile_component(component.as_ref())
                .expect("Cannot compile module"),
        );
        let component = unsafe { Component::deserialize(&engine, &cwasm.as_ref()).unwrap() };

        Self { engine, component }
    }
}

impl Image {
    pub fn init<'a, Res, Req>(&self) -> Result<Container<Res, Req>>
    where
        Res: 'a + Clone + Serialize + Deserialize<'static> + Send + Sync,
        Req: 'a + Clone + Serialize + Deserialize<'static> + Send + Sync,
    {
        let mut linker = Linker::new(&self.engine);
        command::sync::add_to_linker(&mut linker).unwrap();

        let mut wasi = WasiCtxBuilder::new();
        wasi.inherit_stderr();

        let (tx_in, rx_in) = flume::unbounded::<Res>();
        let (tx_out, rx_out) = flume::unbounded::<Req>();

        let input_stream = HostInputStreamBox::<Res> {
            tasks: Default::default(),
        };
        let output_stream = HostOutputStreamBox::<Req> { tx: tx_out };

        let rx = rx_in.clone();
        let tasks = input_stream.tasks.clone();
        std::thread::spawn(move || {
            while let Ok(msg) = rx.recv() {
                tasks.lock().unwrap().push(&msg);
            }
        });

        wasi.stdin(input_stream);
        wasi.stdout(output_stream);

        let wasi = wasi.build();
        let table = Table::new();
        let store = Store::new(&self.engine, WasiContext { wasi, table });

        Ok(Container {
            store: Arc::new(Mutex::new(store)),
            component: self.component.clone(),
            linker,

            tx: tx_in,
            rx: rx_out,
        })
    }
}

impl<Res: Serialize, Req: Serialize> Container<Res, Req> {
    pub fn run(&mut self) -> Result<()> {
        let mut store = self.store.lock().unwrap();
        let (command, _) = Command::instantiate(&mut *store, &self.component, &self.linker)?;

        command
            .wasi_cli_run()
            .call_run(&mut *store)?
            .map_err(|()| anyhow!("guest command returned error"))?;

        Ok(())
    }
}
