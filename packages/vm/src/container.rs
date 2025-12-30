//! Container - A running instance of an Image (like a Docker container)

use anyhow::{Context, Result};
use std::sync::Arc;
use wasmtime::{
    component::{bindgen, Linker},
    Store,
};
use wasmtime_wasi::{ResourceTable, WasiCtx, WasiCtxBuilder, WasiView};

use crate::Image;

bindgen!({
    path: "../../wit",
    world: "tairitsu",
    async: false,
});

use self::tairitsu::core::host_api::Host as HostApiTrait;

/// Type alias for execute handler
type ExecuteHandler = Arc<dyn Fn(String, String) -> Result<String, String> + Send + Sync>;

/// Type alias for log handler
type LogHandler = Arc<dyn Fn(String, String) + Send + Sync>;

/// Host state that implements the host-api interface
pub struct HostState {
    wasi: WasiCtx,
    table: ResourceTable,
    /// Callback for handling execute commands from the guest
    execute_handler: Option<ExecuteHandler>,
    /// Callback for handling log messages from the guest
    log_handler: Option<LogHandler>,
}

impl WasiView for HostState {
    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.wasi
    }

    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }
}

impl HostApiTrait for HostState {
    fn execute(&mut self, command: String, payload: String) -> Result<String, String> {
        if let Some(handler) = &self.execute_handler {
            handler(command, payload)
        } else {
            Err("No execute handler registered".to_string())
        }
    }

    fn log(&mut self, level: String, message: String) {
        if let Some(handler) = &self.log_handler {
            handler(level, message);
        } else {
            eprintln!("[{}] {}", level, message);
        }
    }
}

/// A Container represents a running instance of an Image
/// Similar to Docker containers, it maintains runtime state and can be started/stopped
pub struct Container {
    store: Store<HostState>,
    bindings: Tairitsu,
}

impl Container {
    /// Create a new Container from an Image
    pub fn new(image: &Image) -> Result<Self> {
        let mut wasi = WasiCtxBuilder::new();
        wasi.inherit_stdio().inherit_network();

        let wasi = wasi.build();
        let table = ResourceTable::new();

        let host_state = HostState {
            wasi,
            table,
            execute_handler: None,
            log_handler: None,
        };

        let mut store = Store::new(image.engine(), host_state);

        let mut linker = Linker::new(image.engine());
        wasmtime_wasi::add_to_linker_sync(&mut linker).context("Failed to add WASI to linker")?;

        // Add host API implementation
        Tairitsu::add_to_linker(&mut linker, |state: &mut HostState| state)
            .context("Failed to add host API to linker")?;

        let bindings = Tairitsu::instantiate(&mut store, image.component(), &linker)
            .context("Failed to instantiate component")?;

        Ok(Self { store, bindings })
    }

    /// Set the execute command handler
    pub fn on_execute<F>(&mut self, handler: F) -> &mut Self
    where
        F: Fn(String, String) -> Result<String, String> + Send + Sync + 'static,
    {
        self.store.data_mut().execute_handler = Some(Arc::new(handler));
        self
    }

    /// Set the log message handler
    pub fn on_log<F>(&mut self, handler: F) -> &mut Self
    where
        F: Fn(String, String) + Send + Sync + 'static,
    {
        self.store.data_mut().log_handler = Some(Arc::new(handler));
        self
    }

    /// Initialize the guest module
    pub fn init(&mut self) -> Result<()> {
        self.bindings
            .tairitsu_core_guest_api()
            .call_init(&mut self.store)
            .context("Failed to call guest init")?
            .map_err(|e| anyhow::anyhow!("Guest init failed: {}", e))
    }

    /// Send a command to the guest module
    pub fn handle_command(&mut self, command: &str, payload: &str) -> Result<String> {
        self.bindings
            .tairitsu_core_guest_api()
            .call_handle_command(&mut self.store, command, payload)
            .context("Failed to call guest handle_command")?
            .map_err(|e| anyhow::anyhow!("Guest handle_command failed: {}", e))
    }
}

impl std::fmt::Debug for Container {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Container").finish()
    }
}
