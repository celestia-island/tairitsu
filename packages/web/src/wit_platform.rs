//! WIT-bindings platform implementation for `tairitsu-web`.
//!
//! When the `wit-bindings` Cargo feature is enabled this module provides a
//! [`WitPlatform`] that implements [`tairitsu_vdom::Platform`] by calling the
//! `tairitsu-browser:full` WIT import functions instead of using `web-sys`
//! directly.
//!
//! ## Target requirements
//!
//! The full [`Platform`] implementation is only compiled for `wasm32` targets.
//! On native hosts [`WitPlatform::new`] returns `Err` at runtime, which lets
//! cross-platform code reference the type without a compile-time failure.
//!
//! ## Architecture
//!
//! ```mermaid
//! graph TB
//!     subgraph WASM["WASM Component (wasm32-wasip2)"]
//!         WP["WitPlatform → WIT imports (generated)<br/>node, document, style, event-target"]
//!         BC["BrowserComponent &lt;-- event-callbacks exports"]
//!     end
//!     subgraph HOST["browser-glue (browser / Node.js)"]
//!         BG["dom-glue · events-glue · fetch-glue · …<br/>(packages/browser-glue/src/)"]
//!     end
//!     WP -- "tairitsu-browser:full" --> BG
//!     BG -- "host calls on events" --> BC
//! ```

use anyhow::Result;

#[cfg(feature = "wit-bindings")]
use tairitsu_vdom::{ElementHandle, EventHandle};

// -- Opaque handle wrappers -------------------------------------------------

/// Opaque handle to a DOM node managed by the browser-glue host.
///
/// The inner `u64` is the `node-handle` value assigned by the host via the
/// `tairitsu-browser:full` WIT import.
///
/// Use [`as_raw()`](WitElement::as_raw) to extract the inner host id when needed.
#[cfg(feature = "wit-bindings")]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WitElement(u64);

#[cfg(feature = "wit-bindings")]
impl WitElement {
    /// Construct a new `WitElement` from a raw host id.
    ///
    /// Only platform internals should call this.
    pub const fn from_raw(id: u64) -> Self {
        Self(id)
    }

    /// Extract the raw host id.
    pub const fn as_raw(&self) -> u64 {
        self.0
    }
}

#[cfg(feature = "wit-bindings")]
impl ElementHandle for WitElement {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Opaque handle to a DOM event dispatched by the browser-glue host.
#[cfg(feature = "wit-bindings")]
#[derive(Clone)]
pub struct WitEvent(u64);

#[cfg(feature = "wit-bindings")]
impl WitEvent {
    /// Construct a new `WitEvent` from a raw host id.
    pub const fn from_raw(id: u64) -> Self {
        Self(id)
    }

    /// Extract the raw host id.
    pub const fn as_raw(&self) -> u64 {
        self.0
    }
}

#[cfg(feature = "wit-bindings")]
impl EventHandle for WitEvent {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// -- WitPlatform ------------------------------------------------------------

/// Browser platform backend that calls the `tairitsu-browser:full` WIT world.
///
/// See the [module documentation](self) for the full architecture overview.
///
/// # Feature gate
///
/// Available when the `wit-bindings` Cargo feature is enabled.
///
/// # Target note
///
/// [`tairitsu_vdom::Platform`] is only implemented on `wasm32` targets.
/// On native hosts [`WitPlatform::new`] returns `Err`.
#[derive(Debug, Clone)]
#[cfg(feature = "wit-bindings")]
pub struct WitPlatform;

#[cfg(feature = "wit-bindings")]
impl WitPlatform {
    /// Create a new `WitPlatform`.
    ///
    /// Returns `Err` on non-`wasm32` targets because WIT import functions
    /// require the WebAssembly Component Model linker to supply host
    /// implementations (provided by `packages/browser-glue`).
    pub fn new() -> Result<Self> {
        #[cfg(not(target_family = "wasm"))]
        anyhow::bail!(
            "WitPlatform is only available on wasm32 targets (wasm32-wasip2). \
               Native hosts should execute through tairitsu-runtime/wasmtime host flow."
        );

        #[allow(unreachable_code)]
        Ok(Self)
    }

    /// Set a style property on an element (static method for use in event handlers).
    ///
    /// This is a convenience method that directly calls the WIT style interface
    /// without requiring a Platform instance.
    pub fn set_style_static(_element: &WitElement, _name: &str, _value: &str) -> Result<()> {
        #[cfg(target_family = "wasm")]
        {
            wasm_impl::set_style_static_impl(_element.as_raw(), _name, _value)
        }
        #[cfg(not(target_family = "wasm"))]
        {
            Ok(())
        }
    }

    /// Render a VNode tree into `#app` for WIT-backed browser components.
    ///
    /// This replaces the bootstrap text set by `lifecycle.start` and mounts
    /// the actual app view tree so users can see real UI content.
    pub fn mount_vnode_to_app(&self, _vnode: tairitsu_vdom::VNode) -> Result<()> {
        #[cfg(not(target_family = "wasm"))]
        anyhow::bail!("mount_vnode_to_app is only available on wasm32 targets (wasm32-wasip2)");

        #[cfg(target_family = "wasm")]
        {
            wasm_impl::mount_vnode_to_app(self, _vnode)
        }
    }

    /// Apply a list of patches to update the DOM.
    ///
    /// This method applies patches generated by the diff algorithm to update
    /// the actual DOM, allowing for efficient incremental updates.
    ///
    /// # Arguments
    ///
    /// * `parent` - The parent element to apply patches to
    /// * `patches` - The list of patches to apply
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if all patches were applied successfully, or an error if
    /// any patch application failed.
    ///
    /// # Example
    ///
    /// ```rust
    /// use tairitsu_vdom::VNode;
    /// use tairitsu_web::WitPlatform;
    ///
    /// # fn example(platform: WitPlatform, parent: tairitsu_web::WitElement, old: VNode, new: VNode) -> anyhow::Result<()> {
    /// // Compute the diff between old and new VNode trees
    /// let patches = tairitsu_vdom::diff::diff(Some(&old), &new);
    ///
    /// // Apply the patches to update the DOM
    /// platform.apply_patches(&parent, &patches)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn apply_patches(
        &self,
        _parent: &WitElement,
        _patches: &[tairitsu_vdom::Patch],
    ) -> Result<()> {
        #[cfg(not(target_family = "wasm"))]
        anyhow::bail!("apply_patches is only available on wasm32 targets (wasm32-wasip2)");

        #[cfg(target_family = "wasm")]
        {
            wasm_impl::apply_patches(self, _parent, _patches)
        }
    }

    /// Clear the handle cache.
    ///
    /// Call this when doing large-scale DOM updates or navigation to
    /// invalidate all cached handles.
    ///
    /// # Example
    ///
    /// ```rust
    /// use tairitsu_web::WitPlatform;
    ///
    /// // After navigation or major DOM update
    /// WitPlatform::clear_cache();
    /// ```
    pub fn clear_cache() {
        #[cfg(target_family = "wasm")]
        {
            crate::handle_cache::HandleCache::with(|cache| cache.clear());
        }
    }

    /// Get cache statistics for debugging and monitoring.
    ///
    /// # Returns
    ///
    /// Returns cache statistics including hit rate, number of entries, etc.
    pub fn cache_stats() -> crate::handle_cache::CacheStats {
        #[cfg(target_family = "wasm")]
        {
            crate::handle_cache::HandleCache::with(|cache| cache.stats())
        }
        #[cfg(not(target_family = "wasm"))]
        {
            crate::handle_cache::CacheStats {
                hits: 0,
                misses: 0,
                total: 0,
                hit_rate: 0.0,
                size: 0,
            }
        }
    }

    /// Invalidate cached handles for a specific element.
    ///
    /// Call this when an element's style declaration is replaced or
    /// the element is otherwise modified in a way that invalidates cached handles.
    ///
    /// # Arguments
    ///
    /// * `element` - The element to invalidate caches for
    pub fn invalidate_element_cache(_element: &WitElement) {
        #[cfg(target_family = "wasm")]
        {
            crate::handle_cache::HandleCache::with(|cache| {
                cache.invalidate_style_handle(_element.as_raw());
            });
        }
    }
}

// -- wasm32 Platform implementation ----------------------------------------

#[cfg(all(feature = "wit-bindings", target_family = "wasm"))]
pub mod wasm_impl {
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::sync::atomic::{AtomicU64, Ordering};

    use anyhow::Result;
    use tairitsu_vdom::{
        CanvasContext, CanvasOps, ClipboardOps, ContentEditableOps, DomOps, DomRect, EventData,
        EventWitHandle, FileOps, FocusEvent, GenericEvent, GeoOps, IdbOps, InputEvent,
        KeyboardEvent, LayoutOps, MediaOps, MediaQueryOps, MouseEvent, ObserverOps, QueryOps,
        ScrollOps, TimerOps, VNode,
    };

    use super::{WitElement, WitEvent, WitPlatform};

    thread_local! {
        static CURRENT_RENDER_COMPONENT: RefCell<Option<tairitsu_vdom::ComponentId>> = RefCell::new(None);
    }

    pub fn with_render_component<T>(id: tairitsu_vdom::ComponentId, f: impl FnOnce() -> T) -> T {
        CURRENT_RENDER_COMPONENT.with(|c| {
            *c.borrow_mut() = Some(id);
        });
        let result = f();
        CURRENT_RENDER_COMPONENT.with(|c| {
            *c.borrow_mut() = None;
        });
        result
    }

    fn create_tracked_effect<F>(f: F) -> tairitsu_vdom::EffectHandle
    where
        F: FnMut() + 'static,
    {
        let handle = tairitsu_vdom::create_effect(f);
        CURRENT_RENDER_COMPONENT.with(|c| {
            if let Some(id) = *c.borrow() {
                tairitsu_vdom::register_effect_handle(id, handle.clone());
            }
        });
        handle
    }

    unsafe extern "C" {
        fn tairitsu_component_bootstrap();
    }

    type EventCallback = Box<dyn FnMut(Box<dyn EventData>)>;
    type EventCallbackMap = HashMap<u64, EventCallback>;
    type TimeoutCallback = Option<Box<dyn FnOnce()>>;
    type AnimationCallback = Option<Box<dyn FnOnce(f64)>>;
    type ResizeObserverCallback = Box<dyn FnMut(Vec<tairitsu_vdom::ResizeObserverEntry>)>;
    type MutationObserverCallback = Box<dyn FnMut(Vec<tairitsu_vdom::MutationRecord>)>;
    type MediaQueryListCallback = Box<dyn FnMut(bool)>;
    type ScrollCallback = Box<dyn FnMut(f64, f64)>;
    type WindowResizeCallback = Box<dyn FnMut(i32, i32)>;
    type VideoFrameCallback = Box<dyn FnMut(String)>;

    static NEXT_CALLBACK_ID: AtomicU64 = AtomicU64::new(1);

    pub fn next_callback_id() -> u64 {
        NEXT_CALLBACK_ID.fetch_add(1, Ordering::SeqCst)
    }

    // -- Console helpers -----------------------------------------------------

    /// Log an error.
    ///
    /// For wasm32: The host environment (browser-glue) handles console output.
    /// For native targets: Falls back to stderr.
    #[cfg(target_family = "wasm")]
    fn log_error(message: &str) {
        // No-op for wasm32 - the host environment handles console output
        let _ = message;
    }

    /// Log a warning.
    #[cfg(target_family = "wasm")]
    fn log_warning(message: &str) {
        let _ = message;
    }

    /// Log diagnostic information.
    #[cfg(target_family = "wasm")]
    fn log_info(message: &str) {
        let _ = message;
    }

    /// Log an error (native fallback).
    #[cfg(not(target_family = "wasm"))]
    fn log_error(message: &str) {
        eprintln!("[WitPlatform ERROR] {}", message);
    }

    /// Log a warning (native fallback).
    #[cfg(not(target_family = "wasm"))]
    fn log_warning(message: &str) {
        eprintln!("[WitPlatform WARNING] {}", message);
    }

    /// Log diagnostic information (native fallback).
    #[cfg(not(target_family = "wasm"))]
    fn log_info(message: &str) {
        eprintln!("[WitPlatform] {}", message);
    }

    type GeoCallback =
        Box<dyn FnOnce(Result<tairitsu_vdom::GeoPosition, tairitsu_vdom::GeoPositionError>)>;

    type IntervalCallback = Box<dyn FnMut()>;

    struct WsHandleEntry {
        handle: u64,
        on_open_cb_id: u64,
        on_message_cb_id: u64,
        on_close_cb_id: u64,
        on_error_cb_id: u64,
    }

    thread_local! {
        static EVENT_CALLBACKS: RefCell<EventCallbackMap> = RefCell::new(HashMap::new());
        static ELEMENT_LISTENERS: RefCell<HashMap<(u64, String), u64>>
            = RefCell::new(HashMap::new());
        static TIMEOUT_CALLBACKS: RefCell<HashMap<u64, TimeoutCallback>> = RefCell::new(HashMap::new());
        static INTERVAL_CALLBACKS: RefCell<HashMap<u64, IntervalCallback>> = RefCell::new(HashMap::new());
        pub static ANIMATION_CALLBACKS: RefCell<HashMap<u64, AnimationCallback>> = RefCell::new(HashMap::new());
        static RESIZE_OBSERVER_CALLBACKS: RefCell<HashMap<u64, ResizeObserverCallback>> = RefCell::new(HashMap::new());
        static MUTATION_OBSERVER_CALLBACKS: RefCell<HashMap<u64, MutationObserverCallback>> = RefCell::new(HashMap::new());
        static MEDIA_QUERY_LIST_CALLBACKS: RefCell<HashMap<u64, MediaQueryListCallback>> = RefCell::new(HashMap::new());
        static SCROLL_CALLBACKS: RefCell<HashMap<u64, ScrollCallback>> = RefCell::new(HashMap::new());
        static WINDOW_RESIZE_CALLBACKS: RefCell<HashMap<u64, WindowResizeCallback>> = RefCell::new(HashMap::new());
        static VIDEO_FRAME_CALLBACKS: RefCell<HashMap<u64, VideoFrameCallback>> = RefCell::new(HashMap::new());
        static PROMISE_CALLBACKS: RefCell<HashMap<u64, Box<dyn FnOnce(Result<String, String>)>>> = RefCell::new(HashMap::new());
        static GEO_CALLBACKS: RefCell<HashMap<u64, GeoCallback>> = RefCell::new(HashMap::new());
        static FILE_READER_CALLBACKS: RefCell<HashMap<u64, Box<dyn FnOnce(Result<String, String>)>>> = RefCell::new(HashMap::new());
        static FILE_READER_BIN_CALLBACKS: RefCell<HashMap<u64, Box<dyn FnOnce(Result<Vec<u8>, String>)>>> = RefCell::new(HashMap::new());
        static IDB_CALLBACKS: RefCell<HashMap<u64, Box<dyn FnOnce(Result<String, String>)>>> = RefCell::new(HashMap::new());
        static WS_OPEN_CALLBACKS: RefCell<HashMap<u64, Box<dyn FnOnce()>>> = RefCell::new(HashMap::new());
        static WS_MESSAGE_CALLBACKS: RefCell<HashMap<u64, Box<dyn FnMut(String)>>> = RefCell::new(HashMap::new());
        static WS_CLOSE_CALLBACKS: RefCell<HashMap<u64, Box<dyn FnOnce(u16, String)>>> = RefCell::new(HashMap::new());
        static WS_ERROR_CALLBACKS: RefCell<HashMap<u64, Box<dyn FnOnce()>>> = RefCell::new(HashMap::new());
        static WS_HANDLE_MAP: RefCell<HashMap<u64, WsHandleEntry>> = RefCell::new(HashMap::new());
    }

    // -- WIT binding generation -------------------------------------------

    /// Rust bindings generated from `packages/browser-worlds/wit/browser-full.wit`.
    ///
    /// The `path` is relative to this crate's manifest directory
    /// (`packages/web/`).  `wit_bindgen::generate!` emits
    /// `cargo:rerun-if-changed` directives so the crate is rebuilt whenever
    /// the WIT files change.
    pub mod bindings {
        use super::BrowserComponent;

        include!(concat!(env!("OUT_DIR"), "/wit_bindings_generated.rs"));

        export!(BrowserComponent);
    }

    /// Set a style property on an element (implementation for static method).
    ///
    /// Uses the W3C CSSOM standard interface:
    /// 1. Get the style handle via element-css-inline-style (cached)
    /// 2. Set the property via css-style-declaration
    pub(super) fn set_style_static_impl(element: u64, name: &str, value: &str) -> Result<()> {
        // Try to get style handle from cache first
        let style_handle = crate::handle_cache::HandleCache::with(|cache| {
            if let Some(cached_handle) = cache.get_style_handle(element) {
                return cached_handle;
            }

            // Cache miss - get style handle from WIT interface
            let style_handle =
                bindings::tairitsu_browser::full::element_css_inline_style::get_style(element);

            // Cache it for future use
            cache.set_style_handle(element, style_handle);

            style_handle
        });

        // Set the property using the W3C CSSOM interface
        bindings::tairitsu_browser::full::css_style_declaration::set_property(
            style_handle,
            name,
            value,
            None, // priority (e.g., "important")
        );
        Ok(())
    }

    // -- DOM query helpers (for client-side routing) --------------------

    /// Get the tag name of a DOM element (e.g., `"A"`, `"DIV"`).
    pub fn get_tag_name(_platform: &WitPlatform, element: &WitElement) -> String {
        bindings::tairitsu_browser::full::element::get_tag_name(element.as_raw())
    }

    /// Get an attribute value from a DOM element.
    pub fn get_attribute(
        _platform: &WitPlatform,
        element: &WitElement,
        name: &str,
    ) -> Option<String> {
        bindings::tairitsu_browser::full::element::get_attribute(element.as_raw(), name)
    }

    /// Get the parent element of a DOM node.
    pub fn get_parent_element(_platform: &WitPlatform, element: &WitElement) -> Option<WitElement> {
        bindings::tairitsu_browser::full::node::get_parent_element(element.as_raw())
            .map(WitElement::from_raw)
    }

    /// Prevent default action on an event (via WIT event-target interface).
    pub fn prevent_event_default(event_handle: u64) {
        bindings::tairitsu_browser::full::event_target::prevent_default(event_handle);
    }

    // -- Navigation / Routing helpers (WIT-backed) -----------------------

    pub fn wasm_get_pathname() -> String {
        bindings::tairitsu_browser::full::location::get_pathname()
    }

    pub fn wasm_push_state(url: &str) {
        bindings::tairitsu_browser::full::history::push_state("", "", Some(url));
    }

    pub fn wasm_replace_state(url: &str) {
        bindings::tairitsu_browser::full::history::replace_state("", "", Some(url));
    }

    // -- Component export implementation ---------------------------------

    /// Implements the `event-callbacks` WIT export interface.
    ///
    /// The browser-glue host calls these functions when DOM events fire on
    /// nodes that have a listener registered via
    /// [`WitPlatform::add_event_listener`].  Each callback looks up the
    /// corresponding Rust closure in `EVENT_CALLBACKS` and dispatches the
    /// appropriately typed [`EventData`].
    pub(super) struct BrowserComponent;

    impl bindings::exports::tairitsu_browser::full::event_callbacks::Guest for BrowserComponent {
        fn on_mouse_event(
            listener_id: u64,
            event_handle: u64,
            data: bindings::exports::tairitsu_browser::full::event_callbacks::MouseEventData,
        ) {
            let wit_handle = EventWitHandle::from_wit(event_handle);
            let target = bindings::tairitsu_browser::full::event::get_target(event_handle);
            let current_target =
                bindings::tairitsu_browser::full::event::get_current_target(event_handle);
            let event: Box<dyn EventData> = Box::new(
                MouseEvent::new()
                    .client_x(data.client_x as i32)
                    .client_y(data.client_y as i32)
                    .screen_x(0)
                    .screen_y(0)
                    .offset_x(data.offset_x as i32)
                    .offset_y(data.offset_y as i32)
                    .page_x(0)
                    .page_y(0)
                    .movement_x(0)
                    .movement_y(0)
                    .button(data.button as i16)
                    .buttons(data.buttons as u16)
                    .ctrl_key(data.ctrl_key)
                    .shift_key(data.shift_key)
                    .alt_key(data.alt_key)
                    .meta_key(data.meta_key)
                    .event_handle(wit_handle)
                    .target(target.unwrap_or(0))
                    .current_target(current_target.unwrap_or(0)),
            );
            dispatch_event(listener_id, "mouse", event);
        }

        fn on_keyboard_event(
            listener_id: u64,
            event_handle: u64,
            data: bindings::exports::tairitsu_browser::full::event_callbacks::KeyboardEventData,
        ) {
            let wit_handle = EventWitHandle::from_wit(event_handle);
            let event: Box<dyn EventData> = Box::new(
                KeyboardEvent::new()
                    .key(data.key)
                    .code(data.code)
                    .key_code(data.key_code)
                    .ctrl_key(data.ctrl_key)
                    .shift_key(data.shift_key)
                    .alt_key(data.alt_key)
                    .meta_key(data.meta_key)
                    .repeat(data.repeat)
                    .event_handle(wit_handle),
            );
            dispatch_event(listener_id, "keyboard", event);
        }

        fn on_focus_event(
            listener_id: u64,
            event_handle: u64,
            _data: bindings::exports::tairitsu_browser::full::event_callbacks::FocusEventData,
        ) {
            let wit_handle = EventWitHandle::from_wit(event_handle);
            let event: Box<dyn EventData> = Box::new(FocusEvent::new().event_handle(wit_handle));
            dispatch_event(listener_id, "focus", event);
        }

        fn on_input_event(
            listener_id: u64,
            event_handle: u64,
            data: bindings::exports::tairitsu_browser::full::event_callbacks::InputEventData,
        ) {
            let wit_handle = EventWitHandle::from_wit(event_handle);
            let event: Box<dyn EventData> = Box::new(
                InputEvent::new()
                    .data(data.data.unwrap_or_default())
                    .event_handle(wit_handle),
            );
            dispatch_event(listener_id, "input", event);
        }

        // NOTE: wheel, touch, pointer, transition, and animation events are handled
        // through on_generic_event since the WIT event-callbacks interface doesn't
        // define specialized data structures for them. The event data can be
        // queried via the respective WIT interfaces (wheel-event, touch-event, etc.)
        // using the event_handle.

        fn on_generic_event(listener_id: u64, event_handle: u64, event_type: String) {
            let wit_handle = EventWitHandle::from_wit(event_handle);
            let target = bindings::tairitsu_browser::full::event::get_target(event_handle);

            let event: Box<dyn EventData> = match event_type.as_str() {
                "submit" => {
                    let mut evt = tairitsu_vdom::SubmitEvent::new();
                    evt.target = target;
                    Box::new(evt)
                }
                "change" => {
                    let mut evt = tairitsu_vdom::ChangeEvent::new();
                    evt.target = target;
                    Box::new(evt)
                }
                _ => Box::new(
                    GenericEvent::new()
                        .event_type(&event_type)
                        .event_handle(wit_handle),
                ),
            };
            dispatch_event(listener_id, &event_type, event);
        }
    }

    /// Dispatch an event to the registered callback with error handling.
    fn dispatch_event(listener_id: u64, _event_type: &str, event: Box<dyn EventData>) {
        EVENT_CALLBACKS.with(|m| {
            let mut handler_opt = m.borrow_mut().remove(&listener_id);
            if let Some(handler) = &mut handler_opt {
                handler(event);
            }
            // Re-insert the handler so it can be called again
            if let Some(handler) = handler_opt {
                m.borrow_mut().insert(listener_id, handler);
            }
        });
    }

    impl bindings::exports::tairitsu_browser::full::timer_callbacks::Guest for BrowserComponent {
        fn on_timeout(callback_id: u64) {
            TIMEOUT_CALLBACKS.with(|m| {
                if let Some(callback) = m.borrow_mut().remove(&callback_id) {
                    if let Some(cb) = callback {
                        cb();
                    }
                }
            });
        }

        fn on_interval(callback_id: u64) {
            INTERVAL_CALLBACKS.with(|m| {
                let mut callbacks = m.borrow_mut();
                if let Some(callback) = callbacks.get_mut(&callback_id) {
                    callback();
                }
            });
        }
    }

    impl bindings::exports::tairitsu_browser::full::animation_callbacks::Guest for BrowserComponent {
        fn on_frame(callback_id: u64, timestamp: f64) {
            let callback =
                ANIMATION_CALLBACKS.with(|m| m.borrow_mut().remove(&callback_id).flatten());
            if let Some(cb) = callback {
                cb(timestamp);
            }
        }
    }

    impl bindings::exports::tairitsu_browser::full::resize_observer_callbacks::Guest
        for BrowserComponent
    {
        fn on_resize(
            callback_id: u64,
            entries: Vec<
                bindings::exports::tairitsu_browser::full::resize_observer_callbacks::ResizeEntry,
            >,
        ) {
            RESIZE_OBSERVER_CALLBACKS.with(|m| {
                let mut callbacks = m.borrow_mut();
                if let Some(handler) = callbacks.get_mut(&callback_id) {
                    let converted_entries: Vec<tairitsu_vdom::ResizeObserverEntry> = entries
                        .into_iter()
                        .map(|entry| tairitsu_vdom::ResizeObserverEntry {
                            target: entry.target,
                            content_rect: tairitsu_vdom::DomRect {
                                x: entry.content_rect.x,
                                y: entry.content_rect.y,
                                width: entry.content_rect.width,
                                height: entry.content_rect.height,
                            },
                            border_box_size: vec![],
                            content_box_size: vec![],
                        })
                        .collect();
                    handler(converted_entries);
                }
            });
        }
    }

    impl bindings::exports::tairitsu_browser::full::mutation_observer_callbacks::Guest
        for BrowserComponent
    {
        fn on_mutate(
            callback_id: u64,
            entries: Vec<bindings::exports::tairitsu_browser::full::mutation_observer_callbacks::MutationEntry>,
        ) {
            MUTATION_OBSERVER_CALLBACKS.with(|m| {
                let mut callbacks = m.borrow_mut();
                if let Some(handler) = callbacks.get_mut(&callback_id) {
                    let converted_records: Vec<tairitsu_vdom::MutationRecord> = entries
                        .into_iter()
                        .map(|entry| tairitsu_vdom::MutationRecord {
                            record_type: entry.mutation_type,
                            target: entry.target,
                            added_nodes: vec![],
                            removed_nodes: vec![],
                            previous_sibling: entry.previous_sibling,
                            next_sibling: entry.next_sibling,
                            attribute_name: entry.attribute_name,
                            attribute_namespace: entry.attribute_namespace,
                            old_value: entry.old_value,
                        })
                        .collect();
                    handler(converted_records);
                }
            });
        }
    }

    impl bindings::exports::tairitsu_browser::full::media_query_list_callbacks::Guest
        for BrowserComponent
    {
        fn on_change(callback_id: u64, matches: bool) {
            MEDIA_QUERY_LIST_CALLBACKS.with(|m| {
                let mut callbacks = m.borrow_mut();
                if let Some(handler) = callbacks.get_mut(&callback_id) {
                    handler(matches);
                }
            });
        }
    }

    impl bindings::exports::tairitsu_browser::full::scroll_callbacks::Guest for BrowserComponent {
        fn on_scroll_event(callback_id: u64, scroll_x: f64, scroll_y: f64) {
            SCROLL_CALLBACKS.with(|m| {
                let mut callbacks = m.borrow_mut();
                if let Some(handler) = callbacks.get_mut(&callback_id) {
                    handler(scroll_x, scroll_y);
                }
            });
        }
    }

    impl bindings::exports::tairitsu_browser::full::window_resize_callbacks::Guest
        for BrowserComponent
    {
        fn on_window_resize(callback_id: u64, width: i32, height: i32) {
            WINDOW_RESIZE_CALLBACKS.with(|m| {
                let mut callbacks = m.borrow_mut();
                if let Some(handler) = callbacks.get_mut(&callback_id) {
                    handler(width, height);
                }
            });
        }
    }

    impl bindings::exports::tairitsu_browser::full::video_frame_callbacks::Guest for BrowserComponent {
        fn on_video_frame(callback_id: u64, event: String) {
            VIDEO_FRAME_CALLBACKS.with(|m| {
                let mut callbacks = m.borrow_mut();
                if let Some(handler) = callbacks.get_mut(&callback_id) {
                    handler(event);
                }
            });
        }
    }

    impl bindings::exports::tairitsu_browser::full::promise_callbacks::Guest for BrowserComponent {
        fn on_promise_resolved(promise_id: u64, value: String) {
            PROMISE_CALLBACKS.with(|m| {
                if let Some(callback) = m.borrow_mut().remove(&promise_id) {
                    callback(Ok(value));
                }
            });
        }

        fn on_promise_rejected(promise_id: u64, error: String) {
            PROMISE_CALLBACKS.with(|m| {
                if let Some(callback) = m.borrow_mut().remove(&promise_id) {
                    callback(Err(error));
                }
            });
        }
    }

    impl bindings::exports::tairitsu_browser::full::geolocation_callbacks::Guest for BrowserComponent {
        fn on_position_success(
            callback_id: u64,
            position: bindings::exports::tairitsu_browser::full::geolocation_callbacks::GeoPosition,
        ) {
            GEO_CALLBACKS.with(|m| {
                if let Some(callback) = m.borrow_mut().remove(&callback_id) {
                    callback(Ok(tairitsu_vdom::GeoPosition {
                        latitude: position.latitude,
                        longitude: position.longitude,
                        altitude: position.altitude,
                        accuracy: position.accuracy,
                        altitude_accuracy: position.altitude_accuracy,
                        heading: position.heading,
                        speed: position.speed,
                    }));
                }
            });
        }

        fn on_position_error(
            callback_id: u64,
            error: bindings::exports::tairitsu_browser::full::geolocation_callbacks::GeoPositionError,
        ) {
            GEO_CALLBACKS.with(|m| {
                if let Some(callback) = m.borrow_mut().remove(&callback_id) {
                    callback(Err(tairitsu_vdom::GeoPositionError {
                        code: error.code,
                        message: error.message,
                    }));
                }
            });
        }
    }

    impl bindings::exports::tairitsu_browser::full::idb_callbacks::Guest for BrowserComponent {
        fn on_idb_request_success(callback_id: u64, result: Option<String>) {
            IDB_CALLBACKS.with(|m| {
                if let Some(callback) = m.borrow_mut().remove(&callback_id) {
                    callback(Ok(result.unwrap_or_default()));
                }
            });
        }

        fn on_idb_request_error(callback_id: u64, error: String) {
            IDB_CALLBACKS.with(|m| {
                if let Some(callback) = m.borrow_mut().remove(&callback_id) {
                    callback(Err(error));
                }
            });
        }
    }

    impl bindings::exports::tairitsu_browser::full::file_reader_callbacks::Guest for BrowserComponent {
        fn on_file_reader_load(callback_id: u64, result: String) {
            FILE_READER_CALLBACKS.with(|m| {
                if let Some(callback) = m.borrow_mut().remove(&callback_id) {
                    callback(Ok(result));
                }
            });
        }

        fn on_file_reader_error(callback_id: u64, error: String) {
            FILE_READER_CALLBACKS.with(|m| {
                if let Some(callback) = m.borrow_mut().remove(&callback_id) {
                    callback(Err(error));
                }
            });
        }
    }

    impl bindings::exports::tairitsu_browser::full::web_socket_callbacks::Guest for BrowserComponent {
        fn on_web_socket_open(callback_id: u64) {
            WS_OPEN_CALLBACKS.with(|m| {
                if let Some(callback) = m.borrow_mut().remove(&callback_id) {
                    callback();
                }
            });
        }

        fn on_web_socket_message(callback_id: u64, data: String) {
            WS_MESSAGE_CALLBACKS.with(|m| {
                if let Some(callback) = m.borrow_mut().get_mut(&callback_id) {
                    callback(data);
                }
            });
        }

        fn on_web_socket_close(callback_id: u64, code: u16, reason: String) {
            WS_CLOSE_CALLBACKS.with(|m| {
                if let Some(callback) = m.borrow_mut().remove(&callback_id) {
                    callback(code, reason);
                }
            });
        }

        fn on_web_socket_error(callback_id: u64) {
            WS_ERROR_CALLBACKS.with(|m| {
                if let Some(callback) = m.borrow_mut().remove(&callback_id) {
                    callback();
                }
            });
        }
    }

    impl bindings::exports::tairitsu_browser::full::lifecycle::Guest for BrowserComponent {
        fn start() -> Result<(), String> {
            // Validate the WIT environment before starting the component
            if let Err(e) = validate_wit_environment() {
                let msg = format!("WIT environment validation failed: {}", e);
                log_error(&msg);
                return Err(msg);
            }

            // Register WIT functions for global DOM operations in event handlers
            #[cfg(target_family = "wasm")]
            register_dom_ops_functions();

            unsafe {
                tairitsu_component_bootstrap();
            }

            Ok(())
        }
    }

    /// Register WIT functions for use in event handlers.
    #[cfg(target_family = "wasm")]
    fn register_dom_ops_functions() {
        use tairitsu_vdom::register_wit_functions;

        unsafe {
            register_wit_functions(
                |element, property, value| {
                    let style_handle =
                        bindings::tairitsu_browser::full::element_css_inline_style::get_style(
                            element,
                        );
                    bindings::tairitsu_browser::full::css_style_declaration::set_property(
                        style_handle,
                        property,
                        value,
                        None,
                    );
                    Ok(())
                },
                |element| {
                    let rect = bindings::tairitsu_browser::full::element::get_bounding_client_rect(
                        element,
                    );
                    tairitsu_vdom::DomRect {
                        x: rect.x,
                        y: rect.y,
                        width: rect.width,
                        height: rect.height,
                    }
                },
                |element, name, value| {
                    bindings::tairitsu_browser::full::element::set_attribute(element, name, value);
                },
            );
        }

        use tairitsu_vdom::DomFuncs;

        unsafe {
            tairitsu_vdom::register_dom_functions(DomFuncs {
                get_scroll_top: |el| bindings::tairitsu_browser::full::element::get_scroll_top(el),
                set_scroll_top: |el, v| {
                    bindings::tairitsu_browser::full::element::set_scroll_top(el, v);
                },
                get_scroll_height: |el| {
                    bindings::tairitsu_browser::full::element::get_scroll_height(el)
                },
                get_client_height: |el| {
                    bindings::tairitsu_browser::full::element::get_client_height(el)
                },
                get_class_list: |el| bindings::tairitsu_browser::full::element::get_class_list(el),
                class_list_add: |list, tokens| {
                    bindings::tairitsu_browser::full::dom_token_list::add(list, tokens);
                },
                class_list_remove: |list, tokens| {
                    bindings::tairitsu_browser::full::dom_token_list::remove(list, tokens);
                },
                class_list_contains: |list, token| {
                    bindings::tairitsu_browser::full::dom_token_list::contains(list, token)
                },
                first_child: |el| bindings::tairitsu_browser::full::node::get_first_child(el),
                query_selector_on: |el, sel| {
                    bindings::tairitsu_browser::full::parent_node::query_selector(el, sel)
                },
                create_element: |tag| {
                    bindings::tairitsu_browser::full::document::create_element(tag, None)
                },
                append_child: |parent, child| {
                    bindings::tairitsu_browser::full::node::append_child(parent, child)
                },
                remove_child: |parent, child| {
                    bindings::tairitsu_browser::full::node::remove_child(parent, child)
                },
                get_computed_style_value: |el, prop| {
                    let style =
                        bindings::tairitsu_browser::full::window::get_computed_style(el, None);
                    bindings::tairitsu_browser::full::css_style_declaration::get_property_value(
                        style, prop,
                    )
                },
                set_timeout_fn: |callback: Box<dyn FnOnce()>, ms: i32| -> i32 {
                    let callback_id = next_callback_id() as i32;
                    TIMEOUT_CALLBACKS.with(|m| {
                        m.borrow_mut().insert(callback_id as u64, Some(callback));
                    });
                    bindings::tairitsu_browser::full::platform_helpers::set_timeout(
                        callback_id as u64,
                        ms,
                    ) as i32
                },
                clear_timeout_fn: |id: i32| {
                    TIMEOUT_CALLBACKS.with(|m| {
                        m.borrow_mut().remove(&(id as u64));
                    });
                    bindings::tairitsu_browser::full::platform_helpers::clear_timeout(id);
                },
                set_interval_fn: |callback: Box<dyn FnMut()>, ms: i32| -> i32 {
                    let callback_id = next_callback_id();
                    INTERVAL_CALLBACKS.with(|m| {
                        m.borrow_mut().insert(callback_id, callback);
                    });
                    bindings::tairitsu_browser::full::platform_helpers::set_interval(
                        callback_id,
                        ms,
                    )
                },
                clear_interval_fn: |id: i32| {
                    INTERVAL_CALLBACKS.with(|m| {
                        m.borrow_mut().remove(&(id as u64));
                    });
                    bindings::tairitsu_browser::full::platform_helpers::clear_interval(id);
                },
                request_animation_frame_fn: |callback: Box<dyn FnMut(f64)>| -> u32 {
                    let callback_id = next_callback_id();
                    ANIMATION_CALLBACKS.with(|m| {
                        m.borrow_mut().insert(callback_id, Some(callback));
                    });
                    bindings::tairitsu_browser::full::platform_helpers::request_animation_frame(
                        callback_id,
                    ) as u32
                },
                cancel_animation_frame_fn: |id: u32| {
                    ANIMATION_CALLBACKS.with(|m| {
                        m.borrow_mut().remove(&(id as u64));
                    });
                    bindings::tairitsu_browser::full::platform_helpers::cancel_animation_frame(id);
                },
            });
        }

        tairitsu_vdom::register_ref_resolver(|any| {
            any.downcast_ref::<super::WitElement>().map(|w| w.as_raw())
        });

        log_info("DOM operations functions registered for event handlers");
    }

    /// Validate that the WIT host environment is properly configured.
    fn validate_wit_environment() -> Result<(), String> {
        // Test basic DOM operations to ensure the host is working
        let result = std::panic::catch_unwind(|| {
            // Try to access document - this will call the WIT import
            let _body = bindings::tairitsu_browser::full::document::get_body();
            // Try window operations
            let _width = bindings::tairitsu_browser::full::window::get_inner_width();
            // Note: console operations no longer use WIT interface
            log_info("Environment validation passed");
        });

        match result {
            Ok(_) => Ok(()),
            Err(_) => Err("WIT host call failed during validation".to_string()),
        }
    }

    // -- Platform trait implementation ------------------------------------

    impl DomOps for WitPlatform {
        type Element = WitElement;
        type Event = WitEvent;

        fn create_element(&self, tag: &str) -> Self::Element {
            // New WIT: create-element returns u64 directly, takes optional options
            let handle = bindings::tairitsu_browser::full::document::create_element(tag, None);
            WitElement::from_raw(handle)
        }

        fn create_text_node(&self, text: &str) -> Self::Element {
            // New WIT: create-text-node returns u64 directly
            let handle = bindings::tairitsu_browser::full::document::create_text_node(text);
            WitElement::from_raw(handle)
        }

        fn append_child(&self, parent: &Self::Element, child: &Self::Element) {
            // New WIT: append-child returns u64 (the appended node)
            let _ = bindings::tairitsu_browser::full::node::append_child(
                parent.as_raw(),
                child.as_raw(),
            );
        }

        fn remove_child(&self, parent: &Self::Element, child: &Self::Element) {
            // Invalidate cache for the child element being removed
            crate::handle_cache::HandleCache::with(|cache| {
                cache.invalidate_style_handle(child.as_raw());
            });

            // New WIT: remove-child returns u64 (the removed node)
            let _ = bindings::tairitsu_browser::full::node::remove_child(
                parent.as_raw(),
                child.as_raw(),
            );
        }

        fn set_attribute(&self, element: &Self::Element, name: &str, value: &str) {
            // New WIT: set-attribute returns void
            bindings::tairitsu_browser::full::element::set_attribute(element.as_raw(), name, value);
        }

        fn remove_attribute(&self, element: &Self::Element, name: &str) {
            bindings::tairitsu_browser::full::element::remove_attribute(element.as_raw(), name);
        }

        fn set_style(&self, element: &Self::Element, name: &str, value: &str) {
            // Try to get style handle from cache first
            let style_handle = crate::handle_cache::HandleCache::with(|cache| {
                if let Some(cached_handle) = cache.get_style_handle(element.as_raw()) {
                    return cached_handle;
                }

                // Cache miss - get style handle from WIT interface
                let style_handle =
                    bindings::tairitsu_browser::full::element_css_inline_style::get_style(
                        element.as_raw(),
                    );

                // Cache it for future use
                cache.set_style_handle(element.as_raw(), style_handle);

                style_handle
            });

            // Set the property using the W3C CSSOM interface
            bindings::tairitsu_browser::full::css_style_declaration::set_property(
                style_handle,
                name,
                value,
                None,
            );
        }

        fn set_class(&self, element: &Self::Element, class: &str) {
            bindings::tairitsu_browser::full::element::set_attribute(
                element.as_raw(),
                "class",
                class,
            );
        }

        fn add_event_listener(
            &self,
            element: &Self::Element,
            event: &str,
            handler: Box<dyn FnMut(Box<dyn EventData>)>,
        ) {
            let listener_id = bindings::tairitsu_browser::full::event_target::add_event_listener(
                element.as_raw(),
                event,
                false,
            )
            .unwrap_or_else(|e| {
                log_error(&format!("add_event_listener failed: {}", e));
                0
            });

            EVENT_CALLBACKS.with(|m| m.borrow_mut().insert(listener_id, handler));

            ELEMENT_LISTENERS.with(|m| {
                m.borrow_mut()
                    .insert((element.as_raw(), event.to_string()), listener_id);
            });

            log_info(&format!(
                "Added event listener: event={}, listener_id={}",
                event, listener_id
            ));
        }

        fn remove_event_listener(&self, element: &Self::Element, event: &str) {
            let listener_id = ELEMENT_LISTENERS.with(|m| {
                m.borrow_mut()
                    .remove(&(element.as_raw(), event.to_string()))
            });

            if let Some(listener_id) = listener_id {
                // Remove the callback handler
                EVENT_CALLBACKS.with(|m| m.borrow_mut().remove(&listener_id));
                // Call the W3C remove-event-listener with the listener-id
                let _ = bindings::tairitsu_browser::full::event_target::remove_event_listener(
                    element.as_raw(),
                    listener_id,
                );
                log_info(&format!(
                    "Removed event listener: event={}, listener_id={}",
                    event, listener_id
                ));
            } else {
                log_warning(&format!(
                    "remove_event_listener: no listener found for event '{}' on element {}",
                    event,
                    element.as_raw()
                ));
            }
        }

        fn add_event_listener_with_options(
            &self,
            element: &Self::Element,
            event: &str,
            handler: Box<dyn FnMut(Box<dyn EventData>)>,
            options: tairitsu_vdom::ListenerOptions,
        ) {
            let listener_id = bindings::tairitsu_browser::full::event_target::add_event_listener(
                element.as_raw(),
                event,
                options.capture,
            )
            .unwrap_or_else(|e| {
                log_error(&format!("add_event_listener failed: {}", e));
                0
            });

            EVENT_CALLBACKS.with(|m| m.borrow_mut().insert(listener_id, handler));

            ELEMENT_LISTENERS.with(|m| {
                m.borrow_mut()
                    .insert((element.as_raw(), event.to_string()), listener_id);
            });

            log_info(&format!(
                "Added event listener with options: event={}, listener_id={}",
                event, listener_id
            ));
        }

        fn get_inner_html(&self, element: &Self::Element) -> String {
            bindings::tairitsu_browser::full::element::get_inner_html(element.as_raw())
        }

        fn set_inner_html(&self, element: &Self::Element, html: String) {
            bindings::tairitsu_browser::full::element::set_inner_html(element.as_raw(), &html);
        }

        fn get_attribute(&self, element: &Self::Element, name: &str) -> Option<String> {
            bindings::tairitsu_browser::full::element::get_attribute(element.as_raw(), name)
        }

        fn class_list_add(&self, element: &Self::Element, tokens: &[&str]) {
            let list = bindings::tairitsu_browser::full::element::get_class_list(element.as_raw());
            let wit_tokens: Vec<String> = tokens.iter().map(|s| s.to_string()).collect();
            bindings::tairitsu_browser::full::dom_token_list::add(list, &wit_tokens);
        }

        fn class_list_remove(&self, element: &Self::Element, tokens: &[&str]) {
            let list = bindings::tairitsu_browser::full::element::get_class_list(element.as_raw());
            let wit_tokens: Vec<String> = tokens.iter().map(|s| s.to_string()).collect();
            bindings::tairitsu_browser::full::dom_token_list::remove(list, &wit_tokens);
        }

        fn class_list_contains(&self, element: &Self::Element, token: &str) -> bool {
            let list = bindings::tairitsu_browser::full::element::get_class_list(element.as_raw());
            bindings::tairitsu_browser::full::dom_token_list::contains(list, token)
        }

        fn first_child(&self, element: &Self::Element) -> Option<Self::Element> {
            bindings::tairitsu_browser::full::node::get_first_child(element.as_raw())
                .map(WitElement::from_raw)
        }

        fn insert_before(
            &self,
            parent: &Self::Element,
            new_node: &Self::Element,
            reference_node: Option<&Self::Element>,
        ) {
            let ref_handle = reference_node.map(|r| r.as_raw());
            let _ = bindings::tairitsu_browser::full::node::insert_before(
                parent.as_raw(),
                new_node.as_raw(),
                ref_handle,
            );
        }

        fn query_selector_on(
            &self,
            element: &Self::Element,
            selector: &str,
        ) -> Option<Self::Element> {
            bindings::tairitsu_browser::full::parent_node::query_selector(
                element.as_raw(),
                selector,
            )
            .map(WitElement::from_raw)
        }
    }

    impl TimerOps for WitPlatform {
        fn set_timeout(&self, callback: Box<dyn FnOnce()>, ms: i32) -> i32 {
            let callback_id = next_callback_id();
            TIMEOUT_CALLBACKS.with(|m| m.borrow_mut().insert(callback_id, Some(callback)));
            bindings::tairitsu_browser::full::platform_helpers::set_timeout(callback_id, ms)
        }

        fn clear_timeout(&self, id: i32) {
            bindings::tairitsu_browser::full::platform_helpers::clear_timeout(id)
        }

        fn set_interval(&self, callback: Box<dyn FnMut()>, ms: i32) -> i32 {
            let callback_id = next_callback_id();
            INTERVAL_CALLBACKS.with(|m| m.borrow_mut().insert(callback_id, callback));
            bindings::tairitsu_browser::full::platform_helpers::set_interval(callback_id, ms)
        }

        fn clear_interval(&self, id: i32) {
            bindings::tairitsu_browser::full::platform_helpers::clear_interval(id)
        }

        fn request_animation_frame(&self, callback: Box<dyn FnOnce(f64)>) -> u32 {
            let callback_id = next_callback_id();
            ANIMATION_CALLBACKS.with(|m| m.borrow_mut().insert(callback_id, Some(callback)));
            bindings::tairitsu_browser::full::platform_helpers::request_animation_frame(callback_id)
        }

        fn cancel_animation_frame(&self, id: u32) {
            bindings::tairitsu_browser::full::platform_helpers::cancel_animation_frame(id)
        }
    }

    impl LayoutOps for WitPlatform {
        fn get_bounding_client_rect(&self, element: &Self::Element) -> DomRect {
            let rect = bindings::tairitsu_browser::full::element::get_bounding_client_rect(
                element.as_raw(),
            );
            DomRect {
                x: rect.x,
                y: rect.y,
                width: rect.width,
                height: rect.height,
            }
        }

        fn inner_width(&self) -> i32 {
            bindings::tairitsu_browser::full::window::get_inner_width()
        }

        fn inner_height(&self) -> i32 {
            bindings::tairitsu_browser::full::window::get_inner_height()
        }

        fn get_element_scroll_top(&self, element: &Self::Element) -> f64 {
            bindings::tairitsu_browser::full::element::get_scroll_top(element.as_raw())
        }

        fn set_element_scroll_top(&self, element: &Self::Element, value: f64) {
            bindings::tairitsu_browser::full::element::set_scroll_top(element.as_raw(), value);
        }

        fn get_element_scroll_height(&self, element: &Self::Element) -> i32 {
            bindings::tairitsu_browser::full::element::get_scroll_height(element.as_raw())
        }

        fn get_element_client_height(&self, element: &Self::Element) -> i32 {
            bindings::tairitsu_browser::full::element::get_client_height(element.as_raw())
        }

        fn get_element_client_width(&self, element: &Self::Element) -> i32 {
            bindings::tairitsu_browser::full::element::get_client_width(element.as_raw())
        }
    }

    impl ObserverOps for WitPlatform {
        fn create_resize_observer(
            &self,
            callback: Box<dyn FnMut(Vec<tairitsu_vdom::ResizeObserverEntry>)>,
        ) -> u64 {
            let callback_id = next_callback_id();
            RESIZE_OBSERVER_CALLBACKS.with(|m| m.borrow_mut().insert(callback_id, callback));
            bindings::tairitsu_browser::full::platform_helpers::create_resize_observer(callback_id)
        }

        fn observe_resize(&self, observer: u64, element: &Self::Element) {
            bindings::tairitsu_browser::full::resize_observer::observe(
                observer,
                element.as_raw(),
                None,
            );
        }

        fn unobserve_resize(&self, observer: u64, element: &Self::Element) {
            bindings::tairitsu_browser::full::resize_observer::unobserve(
                observer,
                element.as_raw(),
            );
        }

        fn disconnect_resize(&self, observer: u64) {
            bindings::tairitsu_browser::full::resize_observer::disconnect(observer);
        }

        fn create_mutation_observer(
            &self,
            callback: Box<dyn FnMut(Vec<tairitsu_vdom::MutationRecord>)>,
        ) -> u64 {
            let callback_id = next_callback_id();
            MUTATION_OBSERVER_CALLBACKS.with(|m| m.borrow_mut().insert(callback_id, callback));
            bindings::tairitsu_browser::full::platform_helpers::create_mutation_observer(
                callback_id,
            )
        }

        fn observe_mutations(
            &self,
            observer: u64,
            element: &Self::Element,
            _options: Option<tairitsu_vdom::MutationObserverInit>,
        ) {
            bindings::tairitsu_browser::full::mutation_observer::observe(
                observer,
                element.as_raw(),
                None,
            );
        }

        fn disconnect_mutation(&self, observer: u64) {
            bindings::tairitsu_browser::full::mutation_observer::disconnect(observer);
        }
    }

    impl MediaQueryOps for WitPlatform {
        fn match_media(&self, query: &str) -> u64 {
            bindings::tairitsu_browser::full::window::match_media(query)
        }

        fn media_query_list_get_media(&self, list: u64) -> String {
            bindings::tairitsu_browser::full::media_query_list::get_media(list)
        }

        fn media_query_list_get_matches(&self, list: u64) -> bool {
            bindings::tairitsu_browser::full::media_query_list::get_matches(list)
        }

        fn media_query_list_add_listener(&self, list: u64, callback: Box<dyn FnMut(bool)>) -> u64 {
            let callback_id = next_callback_id();
            MEDIA_QUERY_LIST_CALLBACKS.with(|m| m.borrow_mut().insert(callback_id, callback));
            bindings::tairitsu_browser::full::media_query_list::add_listener(
                list,
                Some(callback_id),
            );
            callback_id
        }

        fn media_query_list_remove_listener(&self, list: u64, listener_id: u64) {
            MEDIA_QUERY_LIST_CALLBACKS.with(|m| m.borrow_mut().remove(&listener_id));
            bindings::tairitsu_browser::full::media_query_list::remove_listener(
                list,
                Some(listener_id),
            );
        }
    }

    impl ClipboardOps for WitPlatform {
        fn copy_to_clipboard(&self, text: &str) -> bool {
            bindings::tairitsu_browser::full::platform_helpers::copy_to_clipboard(text)
        }

        fn read_clipboard(&self) -> Option<String> {
            bindings::tairitsu_browser::full::platform_helpers::read_clipboard()
        }

        fn clipboard_write_text_async(
            &self,
            text: &str,
            on_complete: Box<dyn FnOnce(Result<(), String>)>,
        ) {
            let promise_id =
                bindings::tairitsu_browser::full::platform_helpers::clipboard_write_text_promise(
                    text,
                );
            PROMISE_CALLBACKS.with(|m| {
                m.borrow_mut().insert(
                    promise_id,
                    Box::new(|res| match res {
                        Ok(_) => on_complete(Ok(())),
                        Err(e) => on_complete(Err(e)),
                    }),
                );
            });
        }

        fn clipboard_read_text_async(&self, on_complete: Box<dyn FnOnce(Result<String, String>)>) {
            let promise_id =
                bindings::tairitsu_browser::full::platform_helpers::clipboard_read_text_promise();
            PROMISE_CALLBACKS.with(|m| {
                m.borrow_mut().insert(promise_id, on_complete);
            });
        }
    }

    impl ContentEditableOps for WitPlatform {
        fn get_contenteditable_state(
            &self,
            element: &Self::Element,
        ) -> Option<tairitsu_vdom::ContentEditableState> {
            bindings::tairitsu_browser::full::platform_helpers::get_contenteditable_state(
                element.as_raw(),
            )
            .map(|state| tairitsu_vdom::ContentEditableState {
                editable: state.editable,
                focused: state.focused,
            })
        }

        fn exec_command(&self, command: &str, value: Option<&str>) -> bool {
            bindings::tairitsu_browser::full::document::exec_command(command, None, value)
        }

        fn get_selection_start(&self, element: &Self::Element) -> Option<u32> {
            bindings::tairitsu_browser::full::platform_helpers::get_selection_start(
                element.as_raw(),
            )
        }

        fn get_selection_end(&self, element: &Self::Element) -> Option<u32> {
            bindings::tairitsu_browser::full::platform_helpers::get_selection_end(element.as_raw())
        }

        fn set_content_editable(&self, element: &Self::Element, editable: bool) {
            bindings::tairitsu_browser::full::platform_helpers::set_content_editable(
                element.as_raw(),
                editable,
            );
        }
    }

    impl ScrollOps for WitPlatform {
        fn get_scroll_y(&self) -> f64 {
            bindings::tairitsu_browser::full::window::get_scroll_y()
        }

        fn scroll_to(&self, top: f64, behavior: &str) {
            bindings::tairitsu_browser::full::platform_helpers::scroll_to(top, behavior)
        }

        fn on_scroll(&self, callback: Box<dyn FnMut(f64, f64)>) {
            let callback_id = next_callback_id();
            SCROLL_CALLBACKS.with(|m| m.borrow_mut().insert(callback_id, callback));
            bindings::tairitsu_browser::full::platform_helpers::on_scroll(callback_id);
        }

        fn on_resize(&self, callback: Box<dyn FnMut(i32, i32)>) {
            let callback_id = next_callback_id();
            WINDOW_RESIZE_CALLBACKS.with(|m| m.borrow_mut().insert(callback_id, callback));
            bindings::tairitsu_browser::full::platform_helpers::on_resize_callback(callback_id);
        }

        fn prefers_dark_mode(&self) -> bool {
            bindings::tairitsu_browser::full::platform_helpers::prefers_dark_mode()
        }

        fn request_fullscreen(&self, element: &Self::Element) {
            let _ = bindings::tairitsu_browser::full::element::request_fullscreen(
                element.as_raw(),
                None,
            );
        }

        fn get_scroll_top_from_point(&self, x: i32, y: i32) -> f64 {
            bindings::tairitsu_browser::full::platform_helpers::get_scroll_top_from_point(x, y)
        }

        fn get_scroll_top_by_selector(&self, selector: &str) -> f64 {
            bindings::tairitsu_browser::full::platform_helpers::get_scroll_top_by_selector(selector)
        }

        fn get_target_element_from_event(
            &self,
            client_x: i32,
            client_y: i32,
        ) -> Option<Self::Element> {
            bindings::tairitsu_browser::full::document::element_from_point(
                client_x as f64,
                client_y as f64,
            )
            .map(WitElement::from_raw)
        }
    }

    impl QueryOps for WitPlatform {
        fn get_element_by_id(&self, id: &str) -> Option<Self::Element> {
            bindings::tairitsu_browser::full::platform_helpers::get_element_by_id(id)
                .map(WitElement::from_raw)
        }

        fn query_selector(&self, selector: &str) -> Option<Self::Element> {
            bindings::tairitsu_browser::full::platform_helpers::query_selector(selector)
                .map(WitElement::from_raw)
        }

        fn query_selector_all(&self, selector: &str) -> Vec<Self::Element> {
            bindings::tairitsu_browser::full::platform_helpers::query_selector_all(selector)
                .into_iter()
                .map(WitElement::from_raw)
                .collect()
        }

        fn element_from_point(&self, x: i32, y: i32) -> Option<Self::Element> {
            bindings::tairitsu_browser::full::document::element_from_point(x as f64, y as f64)
                .map(WitElement::from_raw)
        }

        fn element_closest(
            &self,
            element: &Self::Element,
            selector: &str,
        ) -> Option<Self::Element> {
            bindings::tairitsu_browser::full::element::closest(element.as_raw(), selector)
                .map(WitElement::from_raw)
        }

        fn get_element_rect_by_id(&self, id: &str) -> Option<DomRect> {
            bindings::tairitsu_browser::full::platform_helpers::get_element_rect_by_id(id).map(
                |rect| DomRect {
                    x: rect.x,
                    y: rect.y,
                    width: rect.width,
                    height: rect.height,
                },
            )
        }

        fn get_bounding_rect_by_class(
            &self,
            class_name: &str,
            element: &Self::Element,
        ) -> Option<DomRect> {
            bindings::tairitsu_browser::full::platform_helpers::get_bounding_rect_by_class(
                class_name,
                element.as_raw(),
            )
            .map(|rect| DomRect {
                x: rect.x,
                y: rect.y,
                width: rect.width,
                height: rect.height,
            })
        }
    }

    impl CanvasOps for WitPlatform {
        fn get_canvas_context(
            &self,
            element: &Self::Element,
            context_type: &str,
        ) -> Option<CanvasContext> {
            bindings::tairitsu_browser::full::html_canvas_element::get_context(
                element.as_raw(),
                context_type,
                None,
            )
        }

        fn canvas_set_fill_style(&self, ctx: CanvasContext, color: &str) {
            bindings::tairitsu_browser::full::canvas_fill_stroke_styles::set_fill_style(ctx, color)
        }

        fn canvas_fill_rect(&self, ctx: CanvasContext, x: f64, y: f64, w: f64, h: f64) {
            bindings::tairitsu_browser::full::canvas_rect::fill_rect(ctx, x, y, w, h)
        }

        fn canvas_clear_rect(&self, ctx: CanvasContext, x: f64, y: f64, w: f64, h: f64) {
            bindings::tairitsu_browser::full::canvas_rect::clear_rect(ctx, x, y, w, h)
        }

        fn draw_qrcode_on_canvas_by_id(
            &self,
            canvas_id: &str,
            matrix: &[Vec<bool>],
            modules: u64,
            color: &str,
            background: &str,
        ) -> bool {
            let wit_matrix: Vec<Vec<bool>> = matrix.to_vec();
            bindings::tairitsu_browser::full::platform_helpers::draw_qrcode_on_canvas_by_id(
                canvas_id,
                &wit_matrix,
                modules,
                color,
                background,
            )
        }
    }

    impl MediaOps for WitPlatform {
        fn video_play(&self, element: &Self::Element) {
            let _ = bindings::tairitsu_browser::full::html_media_element::play(element.as_raw());
        }

        fn video_pause(&self, element: &Self::Element) {
            bindings::tairitsu_browser::full::html_media_element::pause(element.as_raw());
        }

        fn video_get_current_time(&self, element: &Self::Element) -> f64 {
            bindings::tairitsu_browser::full::html_media_element::get_current_time(element.as_raw())
        }

        fn video_get_duration(&self, element: &Self::Element) -> f64 {
            bindings::tairitsu_browser::full::html_media_element::get_duration(element.as_raw())
        }

        fn video_seek(&self, element: &Self::Element, time: f64) {
            bindings::tairitsu_browser::full::html_media_element::set_current_time(
                element.as_raw(),
                time,
            );
        }

        fn video_set_muted(&self, element: &Self::Element, muted: bool) {
            bindings::tairitsu_browser::full::html_media_element::set_muted(
                element.as_raw(),
                muted,
            );
        }

        fn video_set_volume(&self, element: &Self::Element, volume: f64) {
            bindings::tairitsu_browser::full::html_media_element::set_volume(
                element.as_raw(),
                volume,
            );
        }

        fn create_audio_context(&self) -> u64 {
            bindings::tairitsu_browser::full::platform_helpers::create_audio_context()
        }

        fn create_analyser_node(&self, audio_context: u64) -> u64 {
            bindings::tairitsu_browser::full::base_audio_context::create_analyser(audio_context)
        }

        fn create_media_element_source(&self, audio_context: u64, element: u64) -> u64 {
            bindings::tairitsu_browser::full::audio_context::create_media_element_source(
                audio_context,
                element,
            )
        }

        fn analyser_node_get_frequency_data(&self, analyser: u64) -> Vec<f32> {
            bindings::tairitsu_browser::full::platform_helpers::analyser_get_frequency_data(
                analyser,
            )
        }

        fn analyser_node_get_time_domain_data(&self, analyser: u64) -> Vec<f32> {
            bindings::tairitsu_browser::full::platform_helpers::analyser_get_time_domain_data(
                analyser,
            )
        }
    }

    impl GeoOps for WitPlatform {
        fn get_current_position(
            &self,
            on_success: Box<dyn FnOnce(tairitsu_vdom::GeoPosition)>,
            on_error: Box<dyn FnOnce(tairitsu_vdom::GeoPositionError)>,
            enable_high_accuracy: bool,
            timeout: u32,
            maximum_age: u32,
        ) {
            let callback_id = next_callback_id();
            GEO_CALLBACKS.with(|m| {
                m.borrow_mut().insert(
                    callback_id,
                    Box::new(move |result| match result {
                        Ok(pos) => on_success(pos),
                        Err(err) => on_error(err),
                    }),
                );
            });

            let geo_handle =
                bindings::tairitsu_browser::full::platform_helpers::get_geolocation_handle();
            bindings::tairitsu_browser::full::platform_helpers::get_current_position(
                geo_handle,
                callback_id,
                callback_id,
                enable_high_accuracy,
                timeout,
                maximum_age,
            );
        }
    }

    impl FileOps for WitPlatform {
        fn file_reader_sync_read_as_text(
            &self,
            blob: u64,
            encoding: Option<&str>,
        ) -> Result<String, String> {
            bindings::tairitsu_browser::full::platform_helpers::file_reader_sync_read_as_text(
                blob, encoding,
            )
        }

        fn file_reader_sync_read_as_array_buffer(&self, blob: u64) -> Result<Vec<u8>, String> {
            bindings::tairitsu_browser::full::platform_helpers::file_reader_sync_read_as_array_buffer(blob)
        }

        fn file_reader_read_as_text(
            &self,
            blob: u64,
            encoding: Option<&str>,
            on_complete: Box<dyn FnOnce(Result<String, String>)>,
        ) {
            let callback_id = next_callback_id();
            FILE_READER_CALLBACKS.with(|m| {
                m.borrow_mut().insert(callback_id, on_complete);
            });
            bindings::tairitsu_browser::full::platform_helpers::file_reader_read_as_text(
                blob,
                encoding,
                callback_id,
            );
        }

        fn file_reader_read_as_array_buffer(
            &self,
            blob: u64,
            on_complete: Box<dyn FnOnce(Result<Vec<u8>, String>)>,
        ) {
            let callback_id = next_callback_id();
            FILE_READER_BIN_CALLBACKS.with(|m| {
                m.borrow_mut().insert(
                    callback_id,
                    Box::new(move |result| {
                        on_complete(result.map_err(|e| e));
                    }),
                );
            });
            bindings::tairitsu_browser::full::platform_helpers::file_reader_read_as_array_buffer(
                blob,
                callback_id,
            );
        }
    }

    impl IdbOps for WitPlatform {
        fn idb_open(
            &self,
            name: &str,
            version: Option<u64>,
            on_complete: Box<dyn FnOnce(Result<u64, String>)>,
        ) -> u64 {
            let callback_id = next_callback_id();
            IDB_CALLBACKS.with(|m| {
                m.borrow_mut().insert(
                    callback_id,
                    Box::new(move |result| {
                        let _ = on_complete(result.map(|s| s.parse::<u64>().unwrap_or(0)));
                    }),
                );
            });
            bindings::tairitsu_browser::full::platform_helpers::idb_open(name, version, callback_id)
        }

        fn idb_put(
            &self,
            db: u64,
            store_name: &str,
            value: &str,
            key: Option<&str>,
            on_complete: Box<dyn FnOnce(Result<(), String>)>,
        ) {
            let callback_id = next_callback_id();
            IDB_CALLBACKS.with(|m| {
                m.borrow_mut().insert(
                    callback_id,
                    Box::new(move |result| {
                        let _ = on_complete(result.map(|_| ()));
                    }),
                );
            });
            bindings::tairitsu_browser::full::platform_helpers::idb_put(
                db,
                store_name,
                value,
                key,
                callback_id,
            );
        }

        fn idb_get(
            &self,
            db: u64,
            store_name: &str,
            key: &str,
            on_complete: Box<dyn FnOnce(Result<Option<String>, String>)>,
        ) {
            let callback_id = next_callback_id();
            IDB_CALLBACKS.with(|m| {
                m.borrow_mut().insert(
                    callback_id,
                    Box::new(move |result| {
                        let _ =
                            on_complete(result.map(|s| if s.is_empty() { None } else { Some(s) }));
                    }),
                );
            });
            bindings::tairitsu_browser::full::platform_helpers::idb_get(
                db,
                store_name,
                key,
                callback_id,
            );
        }

        fn idb_delete(
            &self,
            db: u64,
            store_name: &str,
            key: &str,
            on_complete: Box<dyn FnOnce(Result<(), String>)>,
        ) {
            let callback_id = next_callback_id();
            IDB_CALLBACKS.with(|m| {
                m.borrow_mut().insert(
                    callback_id,
                    Box::new(move |result| {
                        let _ = on_complete(result.map(|_| ()));
                    }),
                );
            });
            bindings::tairitsu_browser::full::platform_helpers::idb_delete(
                db,
                store_name,
                key,
                callback_id,
            );
        }

        fn idb_get_all(
            &self,
            db: u64,
            store_name: &str,
            on_complete: Box<dyn FnOnce(Result<Vec<String>, String>)>,
        ) {
            let callback_id = next_callback_id();
            IDB_CALLBACKS.with(|m| {
                m.borrow_mut().insert(
                    callback_id,
                    Box::new(move |result| {
                        let _ = on_complete(result.map(|s| {
                            if s.is_empty() {
                                Vec::new()
                            } else {
                                s.split('\n').map(String::from).collect()
                            }
                        }));
                    }),
                );
            });
            bindings::tairitsu_browser::full::platform_helpers::idb_get_all(
                db,
                store_name,
                callback_id,
            );
        }

        fn idb_clear(
            &self,
            db: u64,
            store_name: &str,
            on_complete: Box<dyn FnOnce(Result<(), String>)>,
        ) {
            let callback_id = next_callback_id();
            IDB_CALLBACKS.with(|m| {
                m.borrow_mut().insert(
                    callback_id,
                    Box::new(move |result| {
                        let _ = on_complete(result.map(|_| ()));
                    }),
                );
            });
            bindings::tairitsu_browser::full::platform_helpers::idb_clear(
                db,
                store_name,
                callback_id,
            );
        }
    }

    pub(super) fn mount_vnode_to_app(platform: &WitPlatform, vnode: VNode) -> Result<()> {
        let doc_handle: u64 = 0;

        let app = if let Some(handle) =
            bindings::tairitsu_browser::full::non_element_parent_node::get_element_by_id(
                doc_handle, "app",
            ) {
            WitElement::from_raw(handle)
        } else {
            let body = bindings::tairitsu_browser::full::document::get_body()
                .ok_or_else(|| anyhow::anyhow!("document.body is not available"))?;
            let div = bindings::tairitsu_browser::full::document::create_element("div", None);
            bindings::tairitsu_browser::full::element::set_attribute(div, "id", "app");
            let _ = bindings::tairitsu_browser::full::node::append_child(body, div);
            WitElement::from_raw(div)
        };

        bindings::tairitsu_browser::full::node::set_text_content(app.as_raw(), Some(""));

        render_vnode(platform, &vnode, &app)
    }

    fn render_vnode(platform: &WitPlatform, vnode: &VNode, parent: &WitElement) -> Result<()> {
        match vnode {
            VNode::Element(velement) => {
                let element = platform.create_element(&velement.tag);

                tairitsu_vdom::runtime::register_element(element.as_raw());

                // Populate element_ref if present
                if let Some(ref element_ref) = velement.element_ref {
                    use std::any::Any;
                    // Store the WitElement in the ref
                    let mut ref_mut = element_ref.borrow_mut();
                    *ref_mut = Some(Box::new(element.clone()) as Box<dyn Any>);
                }

                for (name, value) in &velement.attributes {
                    platform.set_attribute(&element, name, value);
                }

                if !velement.class.static_classes.is_empty() {
                    platform.set_class(&element, &velement.class.static_classes);
                }

                if !velement.style.static_styles.is_empty() {
                    for part in velement.style.static_styles.split(';') {
                        let part = part.trim();
                        if part.is_empty() {
                            continue;
                        }
                        if let Some((name, value)) = part.split_once(':') {
                            platform.set_style(&element, name.trim(), value.trim());
                        }
                    }
                }

                for (name, value) in &velement.style.css_variables {
                    platform.set_style(&element, name, value);
                }

                for (event_name, handler) in &velement.event_handlers {
                    let handler = handler.clone();
                    platform.add_event_listener(
                        &element,
                        event_name,
                        Box::new(move |event| {
                            (handler.borrow_mut())(event);
                        }),
                    );
                }

                if let Some(ref inner) = velement.inner_html {
                    platform.set_inner_html(&element, inner.clone());
                } else {
                    for child in &velement.children {
                        render_vnode(platform, child, &element)?;
                    }
                }

                for (name, compute) in &velement.dynamic_attributes {
                    let raw = element.as_raw();
                    let name = name.clone();
                    let compute = compute.clone();
                    create_tracked_effect(move || {
                        let value = (compute.borrow_mut())();
                        bindings::tairitsu_browser::full::element::set_attribute(
                            raw, &name, &value,
                        );
                    });
                }

                for (name, compute) in &velement.dynamic_styles {
                    let raw = element.as_raw();
                    let name = name.clone();
                    let compute = compute.clone();
                    let platform = platform.clone();
                    create_tracked_effect(move || {
                        let value = (compute.borrow_mut())();
                        let el = WitElement::from_raw(raw);
                        platform.set_style(&el, &name, &value);
                    });
                }

                for compute in &velement.dynamic_classes {
                    let raw = element.as_raw();
                    let compute = compute.clone();
                    create_tracked_effect(move || {
                        let value = (compute.borrow_mut())();
                        bindings::tairitsu_browser::full::element::set_attribute(
                            raw, "class", &value,
                        );
                    });
                }

                platform.append_child(parent, &element);
            }
            VNode::Text(vtext) => {
                let text_node = platform.create_text_node(&vtext.text);
                platform.append_child(parent, &text_node);
            }
            VNode::DynamicText(dt) => {
                let text_node = platform.create_text_node(&dt.initial);
                platform.append_child(parent, &text_node);

                let raw = text_node.as_raw();
                let compute = dt.compute.clone();
                create_tracked_effect(move || {
                    let new_text = (compute.borrow_mut())();
                    bindings::tairitsu_browser::full::node::set_text_content(raw, Some(&new_text));
                });
            }
            VNode::Fragment(children) => {
                for child in children {
                    render_vnode(platform, child, parent)?;
                }
            }
        }

        Ok(())
    }

    /// Apply a list of patches to update the DOM based on the diff result.
    ///
    /// This method takes a parent element and a list of patches, and applies
    /// them to the DOM to reflect the changes from the diff algorithm.
    ///
    /// # Arguments
    ///
    /// * `platform` - The WitPlatform instance for DOM operations
    /// * `parent` - The parent element to apply patches to
    /// * `patches` - The list of patches to apply
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if all patches were applied successfully, or an error if
    /// any patch application failed.
    pub fn apply_patches(
        platform: &WitPlatform,
        parent: &WitElement,
        patches: &[tairitsu_vdom::Patch],
    ) -> Result<()> {
        for patch in patches {
            apply_patch(platform, parent, patch)?;
        }
        Ok(())
    }

    /// Apply a single patch to the DOM.
    ///
    /// This is the core function that handles each type of patch operation.
    fn apply_patch(
        platform: &WitPlatform,
        element: &WitElement,
        patch: &tairitsu_vdom::Patch,
    ) -> Result<()> {
        use tairitsu_vdom::Patch;

        match patch {
            Patch::CreateNode { node } => {
                // Create and append a new node
                render_vnode(platform, node, element)?;
            }

            Patch::RemoveNode => {
                // This is a no-op at this level - removal happens at the parent level
                // via RemoveChild
            }

            Patch::ReplaceNode { node } => {
                // Replace the current element with a new node
                replace_element_content(platform, element, node)?;
            }

            Patch::UpdateText { text } => {
                // Update the text content of the element
                bindings::tairitsu_browser::full::node::set_text_content(
                    element.as_raw(),
                    Some(text),
                );
            }

            Patch::UpdateAttribute { name, value } => {
                platform.set_attribute(element, name, value);
            }

            Patch::AddAttribute { name, value } => {
                platform.set_attribute(element, name, value);
            }

            Patch::RemoveAttribute { name } => {
                platform.remove_attribute(element, name);
            }

            Patch::UpdateStyle { style } => {
                apply_style(platform, element, style)?;
            }

            Patch::UpdateClass { class } => {
                if !class.static_classes.is_empty() {
                    platform.set_class(element, &class.static_classes);
                }
            }

            Patch::InsertChild { index, node } => {
                insert_child_at(platform, element, *index, node)?;
            }

            Patch::RemoveChild { index } => {
                remove_child_at(platform, element, *index)?;
            }

            Patch::UpdateChild {
                index,
                patches: child_patches,
            } => {
                // Get the child at the specified index
                let child = get_child_at(element, *index)?;
                if let Some(child_element) = child {
                    // Apply the nested patches to the child
                    for child_patch in child_patches {
                        apply_patch(platform, &child_element, child_patch)?;
                    }
                }
            }

            Patch::AddEvent { name, handler } => {
                let handler = handler.clone();
                platform.add_event_listener(
                    element,
                    name,
                    Box::new(move |event| {
                        (handler.borrow_mut())(event);
                    }),
                );
            }

            Patch::UpdateEvent { name, handler } => {
                platform.remove_event_listener(element, name);
                let handler = handler.clone();
                platform.add_event_listener(
                    element,
                    name,
                    Box::new(move |event| {
                        (handler.borrow_mut())(event);
                    }),
                );
            }

            Patch::RemoveEvent { name } => {
                platform.remove_event_listener(element, name);
            }

            Patch::MoveChild { from, to } => {
                let child = platform.first_child(element);
                if let Some(_child) = child {
                    tracing::trace!("MoveChild from {} to {} (stub)", from, to);
                }
            }

            Patch::ReorderChildren { removals, moves } => {
                tracing::trace!(
                    "ReorderChildren removals={:?} moves={:?} (stub)",
                    removals,
                    moves
                );
            }
        }

        Ok(())
    }

    /// Create a DOM element from a VNode without attaching it to the DOM.
    fn create_vnode_element(platform: &WitPlatform, vnode: &VNode) -> Result<WitElement> {
        match vnode {
            VNode::Element(velement) => {
                let element = platform.create_element(&velement.tag);

                tairitsu_vdom::runtime::register_element(element.as_raw());

                // Populate element_ref if present (for patch-created elements)
                if let Some(ref element_ref) = velement.element_ref {
                    use std::any::Any;
                    let mut ref_mut = element_ref.borrow_mut();
                    *ref_mut = Some(Box::new(element.clone()) as Box<dyn Any>);
                }

                for (name, value) in &velement.attributes {
                    platform.set_attribute(&element, name, value);
                }

                if !velement.class.static_classes.is_empty() {
                    platform.set_class(&element, &velement.class.static_classes);
                }

                apply_style(platform, &element, &velement.style)?;

                for (event_name, handler) in &velement.event_handlers {
                    let handler = handler.clone();
                    platform.add_event_listener(
                        &element,
                        event_name,
                        Box::new(move |event| {
                            (handler.borrow_mut())(event);
                        }),
                    );
                }

                for (name, compute) in &velement.dynamic_attributes {
                    let raw = element.as_raw();
                    let name = name.clone();
                    let compute = compute.clone();
                    create_tracked_effect(move || {
                        let value = (compute.borrow_mut())();
                        bindings::tairitsu_browser::full::element::set_attribute(
                            raw, &name, &value,
                        );
                    });
                }

                for (name, compute) in &velement.dynamic_styles {
                    let raw = element.as_raw();
                    let name = name.clone();
                    let compute = compute.clone();
                    let p = platform.clone();
                    create_tracked_effect(move || {
                        let value = (compute.borrow_mut())();
                        let el = WitElement::from_raw(raw);
                        p.set_style(&el, &name, &value);
                    });
                }

                for compute in &velement.dynamic_classes {
                    let raw = element.as_raw();
                    let compute = compute.clone();
                    create_tracked_effect(move || {
                        let value = (compute.borrow_mut())();
                        bindings::tairitsu_browser::full::element::set_attribute(
                            raw, "class", &value,
                        );
                    });
                }

                Ok(element)
            }
            VNode::Text(vtext) => Ok(platform.create_text_node(&vtext.text)),
            VNode::DynamicText(dt) => {
                let text_node = platform.create_text_node(&dt.initial);

                let raw = text_node.as_raw();
                let compute = dt.compute.clone();
                create_tracked_effect(move || {
                    let new_text = (compute.borrow_mut())();
                    bindings::tairitsu_browser::full::node::set_text_content(raw, Some(&new_text));
                });

                Ok(text_node)
            }
            VNode::Fragment(_) => {
                // Fragments can't be represented as a single element
                // Create a placeholder div
                Ok(platform.create_element("div"))
            }
        }
    }

    /// Replace the content of an element with a new VNode.
    fn replace_element_content(
        platform: &WitPlatform,
        element: &WitElement,
        vnode: &VNode,
    ) -> Result<()> {
        // Clear existing content
        bindings::tairitsu_browser::full::node::set_text_content(element.as_raw(), Some(""));

        match vnode {
            VNode::Element(velement) => {
                // Update attributes
                for (name, value) in &velement.attributes {
                    platform.set_attribute(element, name, value);
                }

                // Update class
                if !velement.class.static_classes.is_empty() {
                    platform.set_class(element, &velement.class.static_classes);
                }

                // Update style
                apply_style(platform, element, &velement.style)?;

                // Update event listeners (remove old, add new)
                for event_name in velement.event_handlers.keys() {
                    platform.remove_event_listener(element, event_name);
                }
                for (event_name, handler) in &velement.event_handlers {
                    let handler = handler.clone();
                    platform.add_event_listener(
                        &element,
                        event_name,
                        Box::new(move |event| {
                            (handler.borrow_mut())(event);
                        }),
                    );
                }

                // Render children
                for child in &velement.children {
                    render_vnode(platform, child, element)?;
                }
            }
            VNode::Text(vtext) => {
                bindings::tairitsu_browser::full::node::set_text_content(
                    element.as_raw(),
                    Some(&vtext.text),
                );
            }
            VNode::Fragment(children) => {
                for child in children {
                    render_vnode(platform, child, element)?;
                }
            }
            VNode::DynamicText(dt) => {
                bindings::tairitsu_browser::full::node::set_text_content(
                    element.as_raw(),
                    Some(&dt.initial),
                );

                let raw = element.as_raw();
                let compute = dt.compute.clone();
                create_tracked_effect(move || {
                    let new_text = (compute.borrow_mut())();
                    bindings::tairitsu_browser::full::node::set_text_content(raw, Some(&new_text));
                });
            }
        }

        Ok(())
    }

    /// Apply styles to an element.
    fn apply_style(
        platform: &WitPlatform,
        element: &WitElement,
        style: &tairitsu_vdom::Style,
    ) -> Result<()> {
        // Apply static styles
        if !style.static_styles.is_empty() {
            for part in style.static_styles.split(';') {
                let part = part.trim();
                if part.is_empty() {
                    continue;
                }
                if let Some((name, value)) = part.split_once(':') {
                    platform.set_style(element, name.trim(), value.trim());
                }
            }
        }

        // Apply CSS variables
        for (name, value) in &style.css_variables {
            platform.set_style(element, name, value);
        }

        Ok(())
    }

    /// Insert a child node at a specific index.
    fn insert_child_at(
        platform: &WitPlatform,
        parent: &WitElement,
        index: usize,
        node: &VNode,
    ) -> Result<()> {
        // Create the new element
        let new_element = create_vnode_element(platform, node)?;

        // Get the current number of children using child-nodes NodeList
        let child_nodes_handle =
            bindings::tairitsu_browser::full::node::get_child_nodes(parent.as_raw());
        let child_count =
            bindings::tairitsu_browser::full::node_list::get_length(child_nodes_handle) as usize;

        if index >= child_count {
            // Append at the end
            platform.append_child(parent, &new_element);
        } else {
            // Insert at the specific position
            // Get the child at the index to use as the reference node
            let next_sibling =
                bindings::tairitsu_browser::full::node_list::item(child_nodes_handle, index as u32);
            if let Some(sibling) = next_sibling {
                let _ = bindings::tairitsu_browser::full::node::insert_before(
                    parent.as_raw(),
                    new_element.as_raw(),
                    Some(sibling),
                );
            } else {
                platform.append_child(parent, &new_element);
            }
        }

        // If the node is an element, render its children
        if let VNode::Element(velement) = node {
            for child in &velement.children {
                render_vnode(platform, child, &new_element)?;
            }
        }

        Ok(())
    }

    /// Remove a child node at a specific index.
    fn remove_child_at(platform: &WitPlatform, parent: &WitElement, index: usize) -> Result<()> {
        let child = get_child_at(parent, index)?;
        if let Some(child_element) = child {
            tairitsu_vdom::runtime::on_element_removed(child_element.as_raw());
            platform.remove_child(parent, &child_element);
        }
        Ok(())
    }

    /// Get a child element at a specific index.
    fn get_child_at(parent: &WitElement, index: usize) -> Result<Option<WitElement>> {
        let child_nodes_handle =
            bindings::tairitsu_browser::full::node::get_child_nodes(parent.as_raw());
        let child_handle =
            bindings::tairitsu_browser::full::node_list::item(child_nodes_handle, index as u32);
        Ok(child_handle.map(WitElement::from_raw))
    }
}

// -- Tests ---------------------------------------------------------------------

// -- Navigation / Routing helpers ---------------------------------------

/// Get the current URL pathname (e.g., `/components/layer1/button`).
pub fn get_pathname() -> String {
    #[cfg(not(target_family = "wasm"))]
    {
        "/".to_string()
    }
    #[cfg(all(feature = "wit-bindings", target_family = "wasm"))]
    {
        wasm_impl::wasm_get_pathname()
    }
}

/// Push a new URL onto the browser history stack (client-side navigation).
pub fn push_state(url: &str) {
    #[cfg(all(feature = "wit-bindings", target_family = "wasm"))]
    {
        wasm_impl::wasm_push_state(url);
    }
    #[allow(unused_variables)]
    let _ = url;
}

/// Replace the current history entry's URL (client-side navigation).
pub fn replace_state(url: &str) {
    #[cfg(all(feature = "wit-bindings", target_family = "wasm"))]
    {
        wasm_impl::wasm_replace_state(url);
    }
    #[allow(unused_variables)]
    let _ = url;
}

// -- DOM query helpers (public API, delegates to wasm_impl) --------------

/// Get the tag name of a DOM element.
#[cfg(all(feature = "wit-bindings", target_family = "wasm"))]
pub fn get_tag_name(platform: &WitPlatform, element: &WitElement) -> String {
    wasm_impl::get_tag_name(platform, element)
}

/// Get an attribute value from a DOM element.
#[cfg(all(feature = "wit-bindings", target_family = "wasm"))]
pub fn get_attribute(platform: &WitPlatform, element: &WitElement, name: &str) -> Option<String> {
    wasm_impl::get_attribute(platform, element, name)
}

/// Get the parent element of a DOM node.
#[cfg(all(feature = "wit-bindings", target_family = "wasm"))]
pub fn get_parent_element(platform: &WitPlatform, element: &WitElement) -> Option<WitElement> {
    wasm_impl::get_parent_element(platform, element)
}

/// Prevent default action on a DOM event (via WIT event-target interface).
#[cfg(all(feature = "wit-bindings", target_family = "wasm"))]
pub fn prevent_event_default(event_handle: u64) {
    wasm_impl::prevent_event_default(event_handle)
}

#[cfg(all(feature = "wit-bindings", target_family = "wasm"))]
pub struct WsConnection {
    handle: u64,
    on_message_cb_id: u64,
}

#[cfg(all(feature = "wit-bindings", target_family = "wasm"))]
impl WsConnection {
    pub fn send(&self, data: &str) {
        wasm_impl::bindings::tairitsu_browser::full::web_socket::send(self.handle, data);
    }

    pub fn close(&self, code: Option<u16>, reason: Option<&str>) {
        wasm_impl::bindings::tairitsu_browser::full::web_socket::close(
            self.handle,
            code,
            reason.map(|s| s.to_string()),
        );
    }
}

#[cfg(all(feature = "wit-bindings", target_family = "wasm"))]
impl Drop for WsConnection {
    fn drop(&mut self) {
        wasm_impl::WS_HANDLE_MAP.with(|m| {
            m.borrow_mut().remove(&self.handle);
        });
        wasm_impl::WS_MESSAGE_CALLBACKS.with(|m| {
            m.borrow_mut().remove(&self.on_message_cb_id);
        });
    }
}

#[cfg(all(feature = "wit-bindings", target_family = "wasm"))]
pub fn ws_connect(
    url: &str,
    on_open: Box<dyn FnOnce()>,
    on_message: Box<dyn FnMut(String)>,
    on_close: Box<dyn FnOnce(u16, String)>,
    on_error: Box<dyn FnOnce()>,
) -> WsConnection {
    let handle = wasm_impl::bindings::tairitsu_browser::full::platform_helpers::connect_web_socket(
        url.to_string(),
    );

    let on_open_cb_id = wasm_impl::WS_HANDLE_MAP.with(|m| {
        let next = (m.borrow().len() as u64) * 4 + 1;
        let msg_cb_id = next + 1;
        let close_cb_id = next + 2;
        let err_cb_id = next + 3;
        m.borrow_mut().insert(
            handle,
            wasm_impl::WsHandleEntry {
                handle,
                on_open_cb_id: next,
                on_message_cb_id: msg_cb_id,
                on_close_cb_id: close_cb_id,
                on_error_cb_id: err_cb_id,
            },
        );
        next
    });
    let msg_cb_id = on_open_cb_id + 1;
    let close_cb_id = on_open_cb_id + 2;
    let err_cb_id = on_open_cb_id + 3;

    wasm_impl::WS_OPEN_CALLBACKS.with(|m| {
        m.borrow_mut().insert(on_open_cb_id, on_open);
    });
    wasm_impl::WS_MESSAGE_CALLBACKS.with(|m| {
        m.borrow_mut().insert(msg_cb_id, on_message);
    });
    wasm_impl::WS_CLOSE_CALLBACKS.with(|m| {
        m.borrow_mut().insert(close_cb_id, on_close);
    });
    wasm_impl::WS_ERROR_CALLBACKS.with(|m| {
        m.borrow_mut().insert(err_cb_id, on_error);
    });

    WsConnection {
        handle,
        on_message_cb_id: msg_cb_id,
    }
}

#[cfg(all(feature = "wit-bindings", target_family = "wasm"))]
pub fn ws_send(conn: &WsConnection, data: &str) {
    conn.send(data);
}

#[cfg(all(feature = "wit-bindings", target_family = "wasm"))]
pub fn ws_close(conn: &WsConnection, code: Option<u16>, reason: Option<&str>) {
    conn.close(code, reason);
}

#[cfg(all(feature = "wit-bindings", target_family = "wasm"))]
pub fn fetch_text(url: &str, on_complete: Box<dyn FnOnce(Result<String, String>)>) {
    let promise_id = wasm_impl::bindings::tairitsu_browser::full::platform_helpers::fetch_promise(
        url.to_string(),
        None,
    );
    wasm_impl::PROMISE_CALLBACKS.with(|m| {
        m.borrow_mut().insert(promise_id, on_complete);
    });
}

#[cfg(all(feature = "wit-bindings", target_family = "wasm"))]
pub fn fetch_text_with_options(
    url: &str,
    options: &str,
    on_complete: Box<dyn FnOnce(Result<String, String>)>,
) {
    let promise_id = wasm_impl::bindings::tairitsu_browser::full::platform_helpers::fetch_promise(
        url.to_string(),
        Some(options.to_string()),
    );
    wasm_impl::PROMISE_CALLBACKS.with(|m| {
        m.borrow_mut().insert(promise_id, on_complete);
    });
}

#[cfg(test)]
mod tests {
    use tairitsu_vdom::{Classes, Patch, Style, VElement, VNode, VText};

    /// Test that apply_patches handles an empty patch list correctly.
    #[test]
    fn test_apply_empty_patches() {
        // This test verifies that applying an empty list of patches doesn't cause errors.
        // Since we can't actually create DOM elements in unit tests (requires WIT host),
        // we're mainly testing that the function signature and control flow work correctly.
        let patches: Vec<Patch> = vec![];
        assert!(patches.is_empty(), "Empty patches should be empty");
    }

    /// Test that patches can be created and matched correctly.
    #[test]
    fn test_patch_variants() {
        // Test CreateNode patch
        let node = VNode::Text(VText::new("Hello"));
        let create_patch = Patch::CreateNode { node: node.clone() };
        match create_patch {
            Patch::CreateNode { node } => {
                assert_eq!(node, VNode::Text(VText::new("Hello")));
            }
            _ => panic!("Expected CreateNode patch"),
        }

        // Test UpdateText patch
        let update_text_patch = Patch::UpdateText {
            text: "World".to_string(),
        };
        match update_text_patch {
            Patch::UpdateText { text } => {
                assert_eq!(text, "World");
            }
            _ => panic!("Expected UpdateText patch"),
        }

        // Test UpdateAttribute patch
        let update_attr_patch = Patch::UpdateAttribute {
            name: "class".to_string(),
            value: "active".to_string(),
        };
        match update_attr_patch {
            Patch::UpdateAttribute { name, value } => {
                assert_eq!(name, "class");
                assert_eq!(value, "active");
            }
            _ => panic!("Expected UpdateAttribute patch"),
        }

        // Test AddAttribute patch
        let add_attr_patch = Patch::AddAttribute {
            name: "id".to_string(),
            value: "test".to_string(),
        };
        match add_attr_patch {
            Patch::AddAttribute { name, value } => {
                assert_eq!(name, "id");
                assert_eq!(value, "test");
            }
            _ => panic!("Expected AddAttribute patch"),
        }

        // Test RemoveAttribute patch
        let remove_attr_patch = Patch::RemoveAttribute {
            name: "disabled".to_string(),
        };
        match remove_attr_patch {
            Patch::RemoveAttribute { name } => {
                assert_eq!(name, "disabled");
            }
            _ => panic!("Expected RemoveAttribute patch"),
        }

        // Test InsertChild patch
        let child = VNode::Text(VText::new("Child"));
        let insert_child_patch = Patch::InsertChild {
            index: 0,
            node: child,
        };
        match insert_child_patch {
            Patch::InsertChild { index, node } => {
                assert_eq!(index, 0);
                assert_eq!(node, VNode::Text(VText::new("Child")));
            }
            _ => panic!("Expected InsertChild patch"),
        }

        // Test RemoveChild patch
        let remove_child_patch = Patch::RemoveChild { index: 0 };
        match remove_child_patch {
            Patch::RemoveChild { index } => {
                assert_eq!(index, 0);
            }
            _ => panic!("Expected RemoveChild patch"),
        }

        // Test UpdateChild patch
        let child_patches = vec![Patch::UpdateText {
            text: "Updated".to_string(),
        }];
        let update_child_patch = Patch::UpdateChild {
            index: 1,
            patches: child_patches,
        };
        match update_child_patch {
            Patch::UpdateChild { index, patches } => {
                assert_eq!(index, 1);
                assert_eq!(patches.len(), 1);
            }
            _ => panic!("Expected UpdateChild patch"),
        }

        // Test ReplaceNode patch
        let new_node = VNode::Element(VElement::new("div"));
        let replace_patch = Patch::ReplaceNode {
            node: new_node.clone(),
        };
        match replace_patch {
            Patch::ReplaceNode { node } => {
                assert_eq!(node, new_node);
            }
            _ => panic!("Expected ReplaceNode patch"),
        }

        // Test event patches
        #[allow(clippy::type_complexity)]
        let dummy_handler: std::rc::Rc<
            std::cell::RefCell<dyn FnMut(Box<dyn tairitsu_vdom::events::EventData>)>,
        > = std::rc::Rc::new(std::cell::RefCell::new(|_| {}));
        let add_event_patch = Patch::AddEvent {
            name: "click".to_string(),
            handler: dummy_handler.clone(),
        };
        match add_event_patch {
            Patch::AddEvent { name, .. } => {
                assert_eq!(name, "click");
            }
            _ => panic!("Expected AddEvent patch"),
        }

        let update_event_patch = Patch::UpdateEvent {
            name: "click".to_string(),
            handler: dummy_handler.clone(),
        };
        match update_event_patch {
            Patch::UpdateEvent { name, .. } => {
                assert_eq!(name, "click");
            }
            _ => panic!("Expected UpdateEvent patch"),
        }

        let remove_event_patch = Patch::RemoveEvent {
            name: "click".to_string(),
        };
        match remove_event_patch {
            Patch::RemoveEvent { name } => {
                assert_eq!(name, "click");
            }
            _ => panic!("Expected RemoveEvent patch"),
        }
    }

    /// Test nested patches (UpdateChild containing patches).
    #[test]
    fn test_nested_patches() {
        let nested_patches = vec![
            Patch::UpdateAttribute {
                name: "href".to_string(),
                value: "#".to_string(),
            },
            Patch::UpdateClass {
                class: Classes::new().add("active"),
            },
            Patch::UpdateStyle {
                style: Style::new().add("color", "red"),
            },
        ];

        let update_child_patch = Patch::UpdateChild {
            index: 0,
            patches: nested_patches,
        };

        match update_child_patch {
            Patch::UpdateChild { index, patches } => {
                assert_eq!(index, 0);
                assert_eq!(patches.len(), 3);
                // Verify the nested patches are correct
                assert!(matches!(patches[0], Patch::UpdateAttribute { .. }));
                assert!(matches!(patches[1], Patch::UpdateClass { .. }));
                assert!(matches!(patches[2], Patch::UpdateStyle { .. }));
            }
            _ => panic!("Expected UpdateChild patch"),
        }
    }

    /// Test that Patch::is_empty works correctly.
    #[test]
    fn test_patch_is_empty() {
        let empty_patch = Patch::UpdateChild {
            index: 0,
            patches: vec![],
        };
        assert!(empty_patch.is_empty());

        let non_empty_patch = Patch::UpdateChild {
            index: 0,
            patches: vec![Patch::UpdateText {
                text: "test".to_string(),
            }],
        };
        assert!(!non_empty_patch.is_empty());

        // Other patch variants should return false for is_empty
        let text_patch = Patch::UpdateText {
            text: "test".to_string(),
        };
        assert!(!text_patch.is_empty());
    }

    /// Test UpdateStyle and UpdateClass patches
    #[test]
    fn test_style_and_class_patches() {
        // Test UpdateStyle patch
        let style = Style::new()
            .add("color", "red")
            .add("background-color", "blue");
        let update_style_patch = Patch::UpdateStyle { style };
        match update_style_patch {
            Patch::UpdateStyle { style } => {
                assert_eq!(style.static_styles, "color:red;background-color:blue");
            }
            _ => panic!("Expected UpdateStyle patch"),
        }

        // Test UpdateClass patch
        let class = Classes::new().add("btn").add("active");
        let update_class_patch = Patch::UpdateClass { class };
        match update_class_patch {
            Patch::UpdateClass { class } => {
                assert_eq!(class.static_classes, "btn active");
            }
            _ => panic!("Expected UpdateClass patch"),
        }
    }

    /// Test RemoveNode patch
    #[test]
    fn test_remove_node_patch() {
        let remove_node_patch = Patch::RemoveNode;
        match remove_node_patch {
            Patch::RemoveNode => {
                // Success - this is a RemoveNode patch
            }
            _ => panic!("Expected RemoveNode patch"),
        }
    }

    // -- WitElement and WitEvent tests ---------------------------------------

    /// Test WitElement clone behavior.
    #[cfg(feature = "wit-bindings")]
    #[test]
    fn test_wit_element_clone() {
        let element1 = super::WitElement::from_raw(42);
        let element2 = element1;
        let element3 = element2; // WitElement is Copy, so this works

        assert_eq!(element3.as_raw(), 42);
    }

    /// Test WitElement equality and comparison.
    #[cfg(feature = "wit-bindings")]
    #[test]
    fn test_wit_element_equality() {
        let elem1 = super::WitElement::from_raw(100);
        let elem2 = super::WitElement::from_raw(100);
        let elem3 = super::WitElement::from_raw(200);

        assert_eq!(elem1, elem2);
        assert_ne!(elem1, elem3);
    }

    /// Test WitEvent clone behavior.
    #[cfg(feature = "wit-bindings")]
    #[test]
    fn test_wit_event_clone() {
        let event1 = super::WitEvent::from_raw(123);
        let event2 = event1.clone();

        assert_eq!(event1.as_raw(), 123);
        assert_eq!(event2.as_raw(), 123);
    }

    /// Test WitElement implements ElementHandle trait.
    #[cfg(feature = "wit-bindings")]
    #[test]
    fn test_wit_element_element_handle() {
        use tairitsu_vdom::ElementHandle;

        let element = super::WitElement::from_raw(42);
        let any_ref = element.as_any();

        // Should be able to downcast back to WitElement
        assert!(any_ref.is::<super::WitElement>());
        if let Some(downcasted) = any_ref.downcast_ref::<super::WitElement>() {
            assert_eq!(downcasted.as_raw(), 42);
        } else {
            panic!("Failed to downcast to WitElement");
        }
    }

    /// Test WitEvent implements EventHandle trait.
    #[cfg(feature = "wit-bindings")]
    #[test]
    fn test_wit_event_event_handle() {
        use tairitsu_vdom::EventHandle;

        let event = super::WitEvent::from_raw(999);
        let any_ref = event.as_any();

        // Should be able to downcast back to WitEvent
        assert!(any_ref.is::<super::WitEvent>());
        if let Some(downcasted) = any_ref.downcast_ref::<super::WitEvent>() {
            assert_eq!(downcasted.as_raw(), 999);
        } else {
            panic!("Failed to downcast to WitEvent");
        }
    }

    // -- WitPlatform tests ---------------------------------------------------

    /// Test WitPlatform::new() on native targets (should fail).
    #[cfg(all(feature = "wit-bindings", not(target_family = "wasm")))]
    #[test]
    fn test_wit_platform_new_native() {
        let result = super::WitPlatform::new();
        assert!(
            result.is_err(),
            "WitPlatform::new should fail on native targets"
        );

        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("wasm32"), "Error should mention wasm32");
    }

    /// Test WitPlatform::new() on wasm targets (should succeed).
    #[cfg(all(feature = "wit-bindings", target_family = "wasm"))]
    #[test]
    fn test_wit_platform_new_wasm() {
        let result = super::WitPlatform::new();
        assert!(
            result.is_ok(),
            "WitPlatform::new should succeed on wasm targets"
        );
    }

    /// Test WitPlatform::set_style_static on native targets.
    #[cfg(all(feature = "wit-bindings", not(target_family = "wasm")))]
    #[test]
    fn test_set_style_static_native() {
        let element = super::WitElement::from_raw(42);
        let result = super::WitPlatform::set_style_static(&element, "color", "red");

        // On native targets, this should return Ok(()) as a no-op
        assert!(
            result.is_ok(),
            "set_style_static should return Ok on native targets"
        );
    }

    /// Test WitPlatform::set_style_static with various inputs.
    #[cfg(feature = "wit-bindings")]
    #[test]
    fn test_set_style_static_inputs() {
        let element = super::WitElement::from_raw(1);

        // Test with empty values
        let result = super::WitPlatform::set_style_static(&element, "", "");
        #[cfg(target_family = "wasm")]
        {
            // On wasm, this would try to call WIT functions
            // We can't test actual behavior without a host, but we verify it compiles
        }
        #[cfg(not(target_family = "wasm"))]
        {
            assert!(result.is_ok(), "Empty style values should be handled");
        }

        // Test with CSS variable
        let result = super::WitPlatform::set_style_static(&element, "--my-var", "blue");
        #[cfg(not(target_family = "wasm"))]
        {
            assert!(result.is_ok(), "CSS variables should be handled");
        }
    }

    // -- wasm_impl module tests ---------------------------------------------

    /// Test next_callback_id increments.
    #[cfg(all(feature = "wit-bindings", target_family = "wasm"))]
    #[test]
    fn test_next_callback_id() {
        use super::wasm_impl;

        let id1 = wasm_impl::next_callback_id();
        let id2 = wasm_impl::next_callback_id();
        let id3 = wasm_impl::next_callback_id();

        assert!(id2 > id1, "Callback IDs should increment");
        assert!(id3 > id2, "Callback IDs should increment");
        assert_eq!(id2, id1 + 1, "IDs should increment by 1");
        assert_eq!(id3, id2 + 1, "IDs should increment by 1");
    }

    /// Test that log functions don't crash.
    #[cfg(all(feature = "wit-bindings", target_family = "wasm"))]
    #[test]
    fn test_log_functions_no_crash() {
        use super::wasm_impl::{log_error, log_info, log_warning};

        // These should not panic - they're no-ops on wasm
        log_error("test error message");
        log_warning("test warning message");
        log_info("test info message");
    }

    /// Test WitElement Debug output.
    #[cfg(feature = "wit-bindings")]
    #[test]
    fn test_wit_element_debug() {
        let element = super::WitElement::from_raw(42);
        let debug_str = format!("{:?}", element);

        assert!(
            debug_str.contains("42"),
            "Debug output should contain the handle value"
        );
    }

    /// Test WitEvent can be cloned multiple times.
    #[cfg(feature = "wit-bindings")]
    #[test]
    fn test_wit_event_multiple_clones() {
        let event1 = super::WitEvent::from_raw(100);
        let event2 = event1.clone();
        let event3 = event2.clone();

        assert_eq!(event1.as_raw(), 100);
        assert_eq!(event2.as_raw(), 100);
        assert_eq!(event3.as_raw(), 100);
    }

    /// Test WitElement Copy trait behavior.
    #[cfg(feature = "wit-bindings")]
    #[test]
    fn test_wit_element_copy() {
        let elem1 = super::WitElement::from_raw(55);
        let elem2 = elem1; // WitElement is Copy

        // Both should have the same value
        assert_eq!(elem1.as_raw(), 55);
        assert_eq!(elem2.as_raw(), 55);
    }

    /// Test WitElement as_any with different Any operations.
    #[cfg(feature = "wit-bindings")]
    #[test]
    fn test_wit_element_any_operations() {
        use std::any::TypeId;

        use tairitsu_vdom::ElementHandle;

        let element = super::WitElement::from_raw(777);
        let any_ref = element.as_any();

        // Check TypeId
        assert_eq!(any_ref.type_id(), TypeId::of::<super::WitElement>());

        // Verify is<T>() works
        assert!(any_ref.is::<super::WitElement>());
        assert!(!any_ref.is::<String>());
        assert!(!any_ref.is::<i32>());
    }
}
