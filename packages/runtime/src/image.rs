//! Image - Represents a compiled WASM component (like a Docker image)

use anyhow::{Context as AnyhowContext, Result};
use bytes::Bytes;

use wasmtime::{component::Component, error::Context, Config, Engine};
use wit_component::ComponentEncoder;

static WASI_ADAPTER: &[u8] = include_bytes!("../res/wasi_snapshot_preview1.reactor.wasm");

/// An Image represents a compiled WASM component that can be instantiated
/// into one or more Containers. Similar to Docker images, an Image is immutable
/// and can be used to create multiple Container instances.
///
/// For resource-limited execution (fuel, memory, epoch interruption), use
/// [`Image::new_with_config`] or [`Image::from_component_with_config`] with a
/// suitably configured [`Config`].
///
/// # Example — fuel metering
/// ```ignore
/// let mut config = tairitsu::Config::new();
/// config.wasm_component_model(true);
/// config.consume_fuel(true);
///
/// let image = Image::new_with_config(wasm_binary, config)?;
/// let container = Container::builder(image)
///     .with_fuel_limit(1_000_000)
///     .with_guest_initializer(|ctx| { /* ... */ })
///     .build()?;
/// ```
#[derive(Clone)]
pub struct Image {
    engine: Engine,
    component: Component,
}

fn apply_config_defaults(config: &mut Config) {
    config.wasm_component_model(true);
}

impl Image {
    /// Create a new Image from WASM binary with default engine configuration.
    pub fn new(wasm_binary: Bytes) -> Result<Self> {
        Self::new_with_config(wasm_binary, Config::new())
    }

    /// Create a new Image from WASM binary with a custom [`Config`].
    ///
    /// The component-model feature is always enabled regardless of the passed config.
    ///
    /// Use this when you need fuel metering (`consume_fuel`), epoch interruption
    /// (`epoch_interruption`), memory limits (`with_store_limits`), or any other
    /// engine-level configuration.
    pub fn new_with_config(wasm_binary: Bytes, mut config: Config) -> Result<Self> {
        apply_config_defaults(&mut config);

        let engine = Engine::new(&config).context("Failed to create WASM engine")?;

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

    /// Create a new Image from a pre-compiled WIT component binary with default
    /// engine configuration.
    pub fn from_component(component_binary: Bytes) -> Result<Self> {
        Self::from_component_with_config(component_binary, Config::new())
    }

    /// Create a new Image from a pre-compiled WIT component binary with a custom
    /// [`Config`].
    ///
    /// The component-model feature is always enabled regardless of the passed config.
    pub fn from_component_with_config(component_binary: Bytes, mut config: Config) -> Result<Self> {
        apply_config_defaults(&mut config);

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
