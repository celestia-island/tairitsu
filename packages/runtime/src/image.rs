//! Image - Represents a compiled WASM component (like a Docker image)

use anyhow::{Context, Result};
use bytes::Bytes;

use wasmtime::{component::Component, Config, Engine};
use wit_component::ComponentEncoder;

// Include the WASI adapter binary at compile time
static WASI_ADAPTER: &[u8] = include_bytes!("../res/wasi_snapshot_preview1.reactor.wasm");

/// An Image represents a compiled WASM component that can be instantiated
/// into one or more Containers. Similar to Docker images, an Image is immutable
/// and can be used to create multiple Container instances.
#[derive(Clone)]
pub struct Image {
    engine: Engine,
    component: Component,
}

impl Image {
    /// Create a new Image from WASM binary
    ///
    /// # Arguments
    /// * `wasm_binary` - The compiled WASM binary (core module format)
    ///
    /// # Returns
    /// A new Image that can be used to create Containers
    pub fn new(wasm_binary: Bytes) -> Result<Self> {
        let mut config = Config::new();
        config.wasm_component_model(true);
        config.async_support(false);

        let engine = Engine::new(&config).context("Failed to create WASM engine")?;

        // Convert core WASM module to component with WASI adapter
        let component_binary = ComponentEncoder::default()
            .module(wasm_binary.as_ref())
            .context("Failed to parse WASM module")?
            .validate(true)
            .adapter("wasi_snapshot_preview1", WASI_ADAPTER)
            .context("Failed to add WASI adapter")?
            .encode()
            .context("Failed to encode WASM component")?;

        let component = Component::from_binary(&engine, &component_binary)
            .context("Failed to compile WASM component")?;

        Ok(Self { engine, component })
    }

    /// Create a new Image from a WIT component binary
    ///
    /// # Arguments
    /// * `component_binary` - A pre-compiled WIT component binary
    pub fn from_component(component_binary: Bytes) -> Result<Self> {
        let mut config = Config::new();
        config.wasm_component_model(true);
        config.async_support(false);

        let engine = Engine::new(&config).context("Failed to create WASM engine")?;

        let component = Component::from_binary(&engine, component_binary.as_ref())
            .context("Failed to compile WASM component")?;

        Ok(Self { engine, component })
    }

    /// Get the engine used by this image
    pub(crate) fn engine(&self) -> &Engine {
        &self.engine
    }

    /// Get the component
    pub(crate) fn component(&self) -> &Component {
        &self.component
    }
}

impl std::fmt::Debug for Image {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Image").finish()
    }
}
