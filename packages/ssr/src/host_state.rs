//! SSR host state for Wasmtime
//!
//! This module provides the host state implementation used by the SSR container.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use wasmtime_wasi::{ResourceTable, WasiCtx, WasiCtxBuilder, WasiCtxView, WasiView};

use crate::{
    bindings::{DomRect, ResizeObserverEntryHost, ResizeObserverSizeHost},
    virtual_dom::SsrDom,
};

/// Context attached when a guest is stopped by the linear-memory cap.
///
/// Kept here (rather than private to `lib.rs`) because both the trap path and the
/// post-render check report the same condition, and hosts driving their own store
/// through [`SsrHostState::memory_budget_error`] want the same wording.
pub(crate) const MEMORY_CONTEXT: &str =
    "SSR guest exceeded its linear-memory budget; the component did not finish rendering";

/// SSR configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SsrConfig {
    /// Simulated viewport width (default 1920)
    pub viewport_width: i32,
    /// Simulated viewport height (default 1080)
    pub viewport_height: i32,
    /// Current URL path for route-aware rendering (default "/")
    pub current_route: String,
    /// Optional override of [`crate::limits::DEFAULT_FUEL`] for a bounded render.
    ///
    /// `None` (the default) keeps the crate default. This is the knob for hosts
    /// whose components are heavier or lighter than the estimated default; it can
    /// only *change* the budget, not disable it — dropping the bounds stays an
    /// explicit [`crate::RenderOptions::trusted`] opt-in in code.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ssr_fuel: Option<u64>,
    /// Optional override of [`crate::limits::DEFAULT_TIMEOUT`], in milliseconds.
    ///
    /// `None` (the default) keeps the crate default. Like
    /// [`ssr_fuel`](Self::ssr_fuel) this only adjusts a bound that stays in force.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ssr_timeout_ms: Option<u64>,
}

impl Default for SsrConfig {
    fn default() -> Self {
        Self {
            viewport_width: 1920,
            viewport_height: 1080,
            current_route: "/".to_string(),
            ssr_fuel: None,
            ssr_timeout_ms: None,
        }
    }
}

impl SsrConfig {
    /// Create a new config with custom viewport dimensions
    pub fn new(viewport_width: i32, viewport_height: i32) -> Self {
        Self {
            viewport_width,
            viewport_height,
            ..Self::default()
        }
    }

    /// Create a new config with a specific route for route-aware rendering
    pub fn with_route(viewport_width: i32, viewport_height: i32, route: &str) -> Self {
        Self {
            viewport_width,
            viewport_height,
            current_route: if route.is_empty() {
                "/".to_string()
            } else {
                route.to_string()
            },
            ..Self::default()
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
    /// Guest resource limits handed to Wasmtime through [`SsrHostState::limiter_mut`].
    ///
    /// Unbounded by default (a bare `StoreLimits` allows any linear-memory size);
    /// [`crate::limits::apply_limits`] replaces it with a bounded one for untrusted
    /// renders.
    pub store_limits: wasmtime::StoreLimits,
    /// How many `memory.grow` requests the cap denied during this store's lifetime.
    ///
    /// A denied growth is invisible from the outside: the guest sees `memory.grow`
    /// return `-1` and then carries on with a truncated page, returns a typed error,
    /// or traps with `unreachable`. None of those is distinguishable from ordinary
    /// guest behaviour without this counter, which is why the render path checks it
    /// through [`SsrHostState::memory_budget_error`] before returning any HTML.
    pub denied_memory_grows: u64,
}

impl SsrHostState {
    /// Create a new SSR host state with default config
    pub fn new() -> Result<Self> {
        Self::with_config(SsrConfig::default())
    }

    /// Create a new SSR host state with custom config
    pub fn with_config(config: SsrConfig) -> Result<Self> {
        let wasi = WasiCtxBuilder::new().inherit_stdio().build();

        let table = ResourceTable::new();
        let dom = SsrDom::new();
        let store_limits = wasmtime::StoreLimitsBuilder::new().build();

        Ok(Self {
            wasi,
            table,
            dom,
            config,
            store_limits,
            denied_memory_grows: 0,
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

    /// Accessor installed through [`wasmtime::Store::limiter`].
    ///
    /// Keeping the limits in the host state (rather than inside the store) lets
    /// individual renders pick their own bound without rebuilding the engine. The
    /// host state itself is the limiter so a denial can be recorded next to the
    /// limits that produced it.
    pub fn limiter_mut(&mut self) -> &mut dyn wasmtime::ResourceLimiter {
        self
    }

    /// The error to report when this store's guest hit its linear-memory budget.
    ///
    /// `None` while the guest stayed inside the cap. Once any `memory.grow` was
    /// denied the guest did not get the memory it asked for, so whatever DOM it left
    /// behind is a truncation: the render path returns this error instead of
    /// returning that page. Hosts that build their own [`wasmtime::Store`] with
    /// [`crate::limits::apply_limits`] should make the same check after every guest
    /// call.
    pub fn memory_budget_error(&self) -> Option<anyhow::Error> {
        (self.denied_memory_grows > 0).then(|| anyhow::anyhow!(MEMORY_CONTEXT))
    }
}

impl wasmtime::ResourceLimiter for SsrHostState {
    fn memory_growing(
        &mut self,
        current: usize,
        desired: usize,
        maximum: Option<usize>,
    ) -> wasmtime::Result<bool> {
        // `trap_on_grow_failure` is off, so a denied growth is reported to the guest
        // as `memory.grow == -1` (spec behaviour, and what probing allocators
        // expect). Record it here so the host can still tell that the render ran out
        // of its budget.
        let allow = self
            .store_limits
            .memory_growing(current, desired, maximum)?;
        if !allow {
            self.denied_memory_grows = self.denied_memory_grows.saturating_add(1);
        }
        Ok(allow)
    }

    fn memory_grow_failed(&mut self, error: wasmtime::Error) -> wasmtime::Result<()> {
        // Reached when an *allowed* growth still failed (host allocation failure).
        // Unconditionally a budget problem from the render's point of view.
        self.denied_memory_grows = self.denied_memory_grows.saturating_add(1);
        self.store_limits.memory_grow_failed(error)
    }

    fn table_growing(
        &mut self,
        current: usize,
        desired: usize,
        maximum: Option<usize>,
    ) -> wasmtime::Result<bool> {
        self.store_limits.table_growing(current, desired, maximum)
    }

    fn table_grow_failed(&mut self, error: wasmtime::Error) -> wasmtime::Result<()> {
        self.store_limits.table_grow_failed(error)
    }

    fn instances(&self) -> usize {
        self.store_limits.instances()
    }

    fn tables(&self) -> usize {
        self.store_limits.tables()
    }

    fn memories(&self) -> usize {
        self.store_limits.memories()
    }
}

impl Default for SsrHostState {
    fn default() -> Self {
        Self::new().unwrap_or_else(|e| {
            tracing::warn!(
                "Failed to create SsrHostState with default config: {}. Using minimal fallback.",
                e
            );
            Self {
                wasi: WasiCtxBuilder::new().build(),
                table: ResourceTable::new(),
                dom: SsrDom::new(),
                config: SsrConfig::default(),
                store_limits: wasmtime::StoreLimitsBuilder::new().build(),
                denied_memory_grows: 0,
            }
        })
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

// Implement HasData for SsrHostState to support bindgen-generated HostWithStore
impl wasmtime::component::HasData for SsrHostState {
    type Data<'a>
        = &'a mut Self
    where
        Self: 'a;
}

// Implement the bindgen-generated Host traits for resize-observer-entry
// These traits properly handle the dom-rect record type marshaling
impl ResizeObserverEntryHost for SsrHostState {
    fn get_target(&mut self, _self_: u64) -> u64 {
        0
    }

    fn get_content_rect(&mut self, _self_: u64) -> DomRect {
        DomRect {
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
        }
    }

    fn get_border_box_size(&mut self, _self_: u64) -> Vec<u64> {
        Vec::new()
    }

    fn get_content_box_size(&mut self, _self_: u64) -> Vec<u64> {
        Vec::new()
    }

    fn get_device_pixel_content_box_size(&mut self, _self_: u64) -> Vec<u64> {
        Vec::new()
    }
}

impl ResizeObserverSizeHost for SsrHostState {
    fn get_inline_size(&mut self, _self_: u64) -> f64 {
        0.0
    }

    fn get_block_size(&mut self, _self_: u64) -> f64 {
        0.0
    }
}

impl crate::bindings::PlatformHelpersHost for SsrHostState {
    fn set_timeout(&mut self, _callback_id: u64, _ms: i32) -> i32 {
        1
    }

    fn clear_timeout(&mut self, _id: i32) {}

    fn set_interval(&mut self, _callback_id: u64, _ms: i32) -> i32 {
        // SSR has no timer wheel: the render pass is synchronous, so an
        // interval can never fire. Report a handle anyway so a guest that only
        // stores it keeps working; nothing will ever call back.
        1
    }

    fn clear_interval(&mut self, _id: i32) {}

    fn connect_web_socket(&mut self, _url: String) -> u64 {
        // Server-side rendering owns no browser networking: sockets belong to
        // the host that later hydrates the page. A guest that opens one here is
        // out of contract; hand back the neutral handle the other stubs use.
        0
    }

    fn request_animation_frame(&mut self, _callback_id: u64) -> u32 {
        1
    }

    fn cancel_animation_frame(&mut self, _id: u32) {}

    fn create_resize_observer(&mut self, _callback_id: u64) -> u64 {
        1
    }

    fn create_mutation_observer(&mut self, _callback_id: u64) -> u64 {
        1
    }

    fn get_element_by_id(&mut self, _id: String) -> Option<u64> {
        None
    }

    fn query_selector(&mut self, _selector: String) -> Option<u64> {
        None
    }

    fn query_selector_all(&mut self, _selector: String) -> Vec<u64> {
        vec![]
    }

    fn scroll_to(&mut self, _top: f64, _behavior: String) {}

    fn on_scroll(&mut self, _callback_id: u64) {}

    fn on_resize_callback(&mut self, _callback_id: u64) {}

    fn copy_to_clipboard(&mut self, _text: String) -> bool {
        false
    }

    fn read_clipboard(&mut self) -> Option<String> {
        None
    }

    fn clipboard_write_text_promise(&mut self, _text: String) -> u64 {
        0
    }

    fn clipboard_read_text_promise(&mut self) -> u64 {
        0
    }

    fn fetch_promise(&mut self, _url: String, _options: Option<String>) -> u64 {
        0
    }

    fn prefers_dark_mode(&mut self) -> bool {
        false
    }

    fn get_element_rect_by_id(&mut self, _id: String) -> Option<crate::bindings::DomRect> {
        None
    }

    fn get_bounding_rect_by_class(
        &mut self,
        _class_name: String,
        _element: u64,
    ) -> Option<crate::bindings::DomRect> {
        None
    }

    fn get_contenteditable_state(
        &mut self,
        _element: u64,
    ) -> Option<crate::bindings::ContentEditableState> {
        None
    }

    fn set_content_editable(&mut self, _element: u64, _editable: bool) {}

    fn get_selection_start(&mut self, _element: u64) -> Option<u32> {
        None
    }

    fn get_selection_end(&mut self, _element: u64) -> Option<u32> {
        None
    }

    fn create_audio_context(&mut self) -> u64 {
        1
    }

    fn analyser_get_frequency_data(&mut self, _analyser: u64) -> Vec<f32> {
        vec![]
    }

    fn analyser_get_time_domain_data(&mut self, _analyser: u64) -> Vec<f32> {
        vec![]
    }

    fn draw_qrcode_on_canvas_by_id(
        &mut self,
        _canvas_id: String,
        _matrix: Vec<Vec<bool>>,
        _modules: u64,
        _color: String,
        _background: String,
    ) -> bool {
        false
    }

    fn get_scroll_top_from_point(&mut self, _x: i32, _y: i32) -> f64 {
        0.0
    }

    fn get_scroll_top_by_selector(&mut self, _selector: String) -> f64 {
        0.0
    }

    fn get_geolocation_handle(&mut self) -> u64 {
        0
    }

    fn get_current_position(
        &mut self,
        _geo_handle: u64,
        _success_callback_id: u64,
        _error_callback_id: u64,
        _enable_high_accuracy: bool,
        _timeout: u32,
        _maximum_age: u32,
    ) {
    }

    fn file_reader_sync_read_as_text(
        &mut self,
        _blob: u64,
        _encoding: Option<String>,
    ) -> std::result::Result<String, String> {
        Err("FileReader not available in SSR".to_string())
    }

    fn file_reader_sync_read_as_array_buffer(
        &mut self,
        _blob: u64,
    ) -> std::result::Result<Vec<u8>, String> {
        Err("FileReader not available in SSR".to_string())
    }

    fn file_reader_read_as_text(
        &mut self,
        _blob: u64,
        _encoding: Option<String>,
        _callback_id: u64,
    ) {
    }

    fn file_reader_read_as_array_buffer(&mut self, _blob: u64, _callback_id: u64) {}

    fn idb_open(&mut self, _name: String, _version: Option<u64>, _callback_id: u64) -> u64 {
        0
    }

    fn idb_put(
        &mut self,
        _db: u64,
        _store_name: String,
        _value: String,
        _key: Option<String>,
        _callback_id: u64,
    ) {
    }

    fn idb_get(&mut self, _db: u64, _store_name: String, _key: String, _callback_id: u64) {}

    fn idb_delete(&mut self, _db: u64, _store_name: String, _key: String, _callback_id: u64) {}

    fn idb_get_all(&mut self, _db: u64, _store_name: String, _callback_id: u64) {}

    fn idb_clear(&mut self, _db: u64, _store_name: String, _callback_id: u64) {}
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
        assert_eq!(state.config.current_route, "/");
        assert_eq!(state.config.ssr_fuel, None);
        assert_eq!(state.config.ssr_timeout_ms, None);
        assert_eq!(state.denied_memory_grows, 0);
        assert!(state.memory_budget_error().is_none());
    }

    #[test]
    fn test_host_state_with_config() {
        let config = SsrConfig::with_route(1280, 720, "/about");
        let state = SsrHostState::with_config(config).unwrap();
        assert_eq!(state.config.viewport_width, 1280);
        assert_eq!(state.config.viewport_height, 720);
        assert_eq!(state.config.current_route, "/about");
    }
}
