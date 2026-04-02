//! SSR host state for Wasmtime
//!
//! This module provides the host state implementation used by the SSR container.

use wasmtime_wasi::{ResourceTable, WasiCtx, WasiCtxBuilder, WasiCtxView, WasiView};

use crate::{
    bindings::{DomRect, ResizeObserverEntryHost, ResizeObserverSizeHost},
    virtual_dom::SsrDom,
};

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
        let wasi = WasiCtxBuilder::new().inherit_stdio().build();

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

// Implement the platform-helpers Host trait
impl crate::bindings::PlatformHelpersHost for SsrHostState {
    fn get_bounding_client_rect(&mut self, _element: u64) -> crate::bindings::DomRect {
        crate::bindings::DomRect {
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
        }
    }

    fn inner_width(&mut self) -> i32 {
        self.dom.viewport_width()
    }

    fn inner_height(&mut self) -> i32 {
        self.dom.viewport_height()
    }

    fn set_timeout(&mut self, _callback_id: u64, _ms: i32) -> i32 {
        1
    }

    fn clear_timeout(&mut self, _id: i32) {}

    fn request_animation_frame(&mut self, _callback_id: u64) -> u32 {
        1
    }

    fn cancel_animation_frame(&mut self, _id: u32) {}

    fn create_resize_observer(&mut self, _callback_id: u64) -> u64 {
        1
    }

    fn observe_resize(&mut self, _observer: u64, _element: u64) {}

    fn unobserve_resize(&mut self, _observer: u64, _element: u64) {}

    fn disconnect_resize(&mut self, _observer: u64) {}

    fn create_mutation_observer(&mut self, _callback_id: u64) -> u64 {
        1
    }

    fn observe_mutations(&mut self, _observer: u64, _element: u64, _options: Option<u64>) {}

    fn disconnect_mutation(&mut self, _observer: u64) {}

    fn get_element_by_id(&mut self, _id: String) -> Option<u64> {
        None
    }

    fn query_selector(&mut self, _selector: String) -> Option<u64> {
        None
    }

    fn query_selector_all(&mut self, _selector: String) -> Vec<u64> {
        vec![]
    }

    fn element_from_point(&mut self, _x: i32, _y: i32) -> Option<u64> {
        None
    }

    fn element_closest(&mut self, _element: u64, _selector: String) -> Option<u64> {
        None
    }

    fn get_scroll_y(&mut self) -> f64 {
        0.0
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

    fn request_fullscreen(&mut self, _element: u64) {}

    fn get_contenteditable_state(
        &mut self,
        _element: u64,
    ) -> Option<crate::bindings::ContentEditableState> {
        None
    }

    fn set_content_editable(&mut self, _element: u64, _editable: bool) {}

    fn exec_command(&mut self, _command: String, _value: Option<String>) -> bool {
        false
    }

    fn get_selection_start(&mut self, _element: u64) -> Option<u32> {
        None
    }

    fn get_selection_end(&mut self, _element: u64) -> Option<u32> {
        None
    }

    fn create_audio_context(&mut self) -> u64 {
        1
    }

    fn create_analyser_node(&mut self, _audio_context: u64) -> u64 {
        1
    }

    fn create_media_element_source(&mut self, _audio_context: u64, _element: u64) -> u64 {
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
