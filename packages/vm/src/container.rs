//! Container - A running instance of an Image (like a Docker container)

use anyhow::{Context, Result};
use std::sync::Arc;
use wasmtime::{
    component::{bindgen, Linker},
    Store,
};
use wasmtime_wasi::{ResourceTable, WasiCtx, WasiCtxBuilder, WasiView};

use crate::commands::{
    deserialize_command, serialize_command, GuestCommands, GuestResponse, HostCommands,
    HostResponse, LogLevel,
};
use crate::Image;

bindgen!({
    path: "../../wit",
    world: "tairitsu",
    async: false,
});

use self::tairitsu::core::host_api::Host as HostApiTrait;

/// Type alias for execute handler with typed commands
type ExecuteHandler = Arc<dyn Fn(HostCommands) -> Result<HostResponse, String> + Send + Sync>;

/// Type alias for log handler
type LogHandler = Arc<dyn Fn(LogLevel, String) + Send + Sync>;

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
        // Deserialize the command from JSON
        let cmd: HostCommands = deserialize_command(&command).or_else(|_| {
            // Fallback for legacy string-based commands
            Ok::<HostCommands, String>(HostCommands::Custom {
                name: command.clone(),
                data: payload.clone(),
            })
        })?;

        if let Some(handler) = &self.execute_handler {
            let response = handler(cmd)?;
            serialize_command(&response)
        } else {
            Err("No execute handler registered".to_string())
        }
    }

    fn log(&mut self, level: String, message: String) {
        let log_level = LogLevel::from(level.as_str());
        if let Some(handler) = &self.log_handler {
            handler(log_level, message);
        } else {
            eprintln!("[{}] {}", log_level, message);
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

    /// Set the execute command handler with typed commands
    pub fn on_execute<F>(&mut self, handler: F) -> &mut Self
    where
        F: Fn(HostCommands) -> Result<HostResponse, String> + Send + Sync + 'static,
    {
        self.store.data_mut().execute_handler = Some(Arc::new(handler));
        self
    }

    /// Set the log message handler with typed log levels
    pub fn on_log<F>(&mut self, handler: F) -> &mut Self
    where
        F: Fn(LogLevel, String) + Send + Sync + 'static,
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

    /// Send a typed command to the guest module
    pub fn send_command(&mut self, command: GuestCommands) -> Result<GuestResponse> {
        let cmd_str = serialize_command(&command)
            .map_err(|e| anyhow::anyhow!("Serialization error: {}", e))?;
        let payload = String::new(); // Payload is embedded in the command

        let response_str = self
            .bindings
            .tairitsu_core_guest_api()
            .call_handle_command(&mut self.store, &cmd_str, &payload)
            .context("Failed to call guest handle_command")?
            .map_err(|e| anyhow::anyhow!("Guest handle_command failed: {}", e))?;

        deserialize_command(&response_str)
            .map_err(|e| anyhow::anyhow!("Failed to deserialize response: {}", e))
    }

    /// Send a command to the guest module (legacy string-based interface)
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
