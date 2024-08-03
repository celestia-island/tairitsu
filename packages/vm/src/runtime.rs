use anyhow::{Context, Result};
use bytes::Bytes;
use flume::{Receiver, Sender};
use lazy_static::lazy_static;
use std::sync::{Arc, Mutex};

use wasmtime::{
    component::{Component, Linker},
    Config, Engine, Store,
};
use wasmtime_wasi::{
    add_to_linker_sync, bindings::sync::Command, DirPerms, FilePerms, ResourceTable, WasiCtx,
    WasiCtxBuilder, WasiView,
};
use wit_component::ComponentEncoder;

use crate::stream::{HostInputStreamBox, HostOutputStreamBox};
use tairitsu_utils::types::proto::backend::Msg;

lazy_static! {
    static ref ADAPTER: Bytes =
        Bytes::from_static(include_bytes!("../res/wasi_snapshot_preview1.command.wasm"));
}

pub struct WasiContext {
    wasi: WasiCtx,
    table: ResourceTable,
}

impl WasiView for WasiContext {
    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.wasi
    }
    fn table(&mut self) -> &mut ResourceTable {
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
pub struct Container {
    pub store: Arc<Mutex<Store<WasiContext>>>,
    pub component: Component,
    pub linker: Linker<WasiContext>,

    pub tx: Sender<Msg>,
    pub rx: Receiver<Msg>,
}

impl std::fmt::Debug for Container {
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
        let component = unsafe { Component::deserialize(&engine, cwasm.as_ref()).unwrap() };

        Self { engine, component }
    }

    pub fn init(&self) -> Result<Container> {
        let mut linker = Linker::new(&self.engine);
        add_to_linker_sync(&mut linker)?;

        let mut wasi = WasiCtxBuilder::new();
        wasi.inherit_stderr();

        let (tx_in, rx_in) = flume::unbounded();
        let (tx_out, rx_out) = flume::unbounded();

        let input_stream = HostInputStreamBox {
            tasks: Default::default(),
        };
        let output_stream = HostOutputStreamBox { tx: tx_out };

        let rx = rx_in.clone();
        let tasks = input_stream.tasks.clone();
        std::thread::spawn(move || {
            while let Ok(msg) = rx.recv() {
                tasks.lock().unwrap().push(msg);
            }
        });

        wasi.stdin(input_stream);
        wasi.stdout(output_stream);

        // TODO - This is a temporary solution to make the example work

        wasi.preopened_dir("./target/tmp", "/tmp", DirPerms::all(), FilePerms::all())?;
        wasi.inherit_network();
        wasi.allow_ip_name_lookup(true);
        wasi.allow_tcp(true);
        wasi.allow_udp(true);

        let wasi = wasi.build();
        let table = ResourceTable::new();
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

impl Container {
    pub fn run(&mut self) -> Result<()> {
        let mut store = self.store.lock().unwrap();
        let command = Command::instantiate(&mut *store, &self.component, &mut self.linker)?;

        let _ = command
            .wasi_cli_run()
            .call_run(&mut *store)
            .context("Failed to run the command")?;

        Ok(())
    }
}
