//! SSR host state for Wasmtime
//!
//! This module provides the host state implementation used by the SSR container.

use wasmtime_wasi::{ResourceTable, WasiCtx, WasiCtxBuilder, WasiCtxView, WasiView};

use crate::virtual_dom::SsrDom;

/// SSR configuration
#[derive(Debug, Clone)]
pub struct SsrConfig {
    /// Simulated viewport width (default 1920)
    pub viewport_width: i32,
    /// Simulated viewport height (default 1080)
    pub viewport_height: i32,
}

impl Default for SsrConfig {
    fn default() -> Self {
        Self {
            viewport_width: 1920,
            viewport_height: 1080,
        }
    }
}

impl SsrConfig {
    /// Create a new config with custom viewport dimensions
    pub fn new(viewport_width: i32, viewport_height: i32) -> Self {
        Self {
            viewport_width,
            viewport_height,
        }
    }
}

/// SSR host state
///
/// This combines WASI support with the in-memory DOM for SSR.
pub struct SsrHostState {
    wasi: WasiCtx,
    table: ResourceTable,
    pub dom: SsrDom,
    pub config: SsrConfig,
}

impl SsrHostState {
    /// Create a new SSR host state with default config
    pub fn new() -> anyhow::Result<Self> {
        Self::with_config(SsrConfig::default())
    }

    /// Create a new SSR host state with custom config
    pub fn with_config(config: SsrConfig) -> anyhow::Result<Self> {
        let wasi = WasiCtxBuilder::new()
            .inherit_stdio()
            .build();

        let table = ResourceTable::new();
        let dom = SsrDom::new();

        Ok(Self {
            wasi,
            table,
            dom,
            config,
        })
    }

    /// Get mutable reference to the DOM
    pub fn dom_mut(&mut self) -> &mut SsrDom {
        &mut self.dom
    }

    /// Get reference to the DOM
    pub fn dom_ref(&self) -> &SsrDom {
        &self.dom
    }
}

impl Default for SsrHostState {
    fn default() -> Self {
        Self::new().expect("Failed to create SsrHostState")
    }
}

impl WasiView for SsrHostState {
    fn ctx(&mut self) -> wasmtime_wasi::WasiCtxView<'_> {
        WasiCtxView {
            ctx: &mut self.wasi,
            table: &mut self.table,
        }
    }
}

impl tairitsu::container::HostStateImpl for SsrHostState {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_host_state_creation() {
        let state = SsrHostState::new().unwrap();
        assert_ne!(state.dom.body_handle(), 0);
        assert_eq!(state.config.viewport_width, 1920);
        assert_eq!(state.config.viewport_height, 1080);
    }

    #[test]
    fn test_host_state_with_config() {
        let config = SsrConfig::new(1280, 720);
        let state = SsrHostState::with_config(config).unwrap();
        assert_eq!(state.config.viewport_width, 1280);
        assert_eq!(state.config.viewport_height, 720);
    }
}
