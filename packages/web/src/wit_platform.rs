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
//! ```text
//! ┌──────────────────────────────────────────────────┐  wasm32-wasip2
//! │                  WASM Component                  │
//! │                                                  │
//! │  WitPlatform ──► WIT imports (generated)         │
//! │      node, document, style, event-target         │──► tairitsu-browser:full
//! │                                                  │
//! │  BrowserComponent ◄── event-callbacks exports    │◄── host calls on events
//! └──────────────────────────────────────────────────┘
//!          ▲ component-model import / export boundary
//! ┌────────┴─────────────────────────────────────────┐  browser / Node.js
//! │  browser-glue  (packages/browser-glue/src/)      │
//! │  dom-glue · events-glue · fetch-glue · …         │
//! └──────────────────────────────────────────────────┘
//! ```

use anyhow::Result;

#[cfg(feature = "wit-bindings")]
use tairitsu_vdom::{ElementHandle, EventHandle};

// ── Opaque handle wrappers ─────────────────────────────────────────────────

/// Opaque handle to a DOM node managed by the browser-glue host.
///
/// The inner `u64` is the `node-handle` value assigned by the host via the
/// `tairitsu-browser:full` WIT import.
#[cfg(feature = "wit-bindings")]
#[derive(Clone)]
pub struct WitElement(pub u64);

#[cfg(feature = "wit-bindings")]
impl ElementHandle for WitElement {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Opaque handle to a DOM event dispatched by the browser-glue host.
#[cfg(feature = "wit-bindings")]
#[derive(Clone)]
pub struct WitEvent(pub u64);

#[cfg(feature = "wit-bindings")]
impl EventHandle for WitEvent {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// ── WitPlatform ────────────────────────────────────────────────────────────

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
    pub fn set_style_static(element: &WitElement, name: &str, value: &str) -> Result<()> {
        #[cfg(target_family = "wasm")]
        {
            wasm_impl::set_style_static_impl(element.0, name, value)
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
    pub fn mount_vnode_to_app(&self, _vnode: &tairitsu_vdom::VNode) -> Result<()> {
        #[cfg(not(target_family = "wasm"))]
        anyhow::bail!("mount_vnode_to_app is only available on wasm32 targets (wasm32-wasip2)");

        #[cfg(target_family = "wasm")]
        {
            wasm_impl::mount_vnode_to_app(self, _vnode)
        }
    }
}

// ── wasm32 Platform implementation ────────────────────────────────────────
//
// Everything below is compiled only when *both*:
//   • the `wit-bindings` Cargo feature is active, and
//   • the target is a wasm32 family (component model capable).
//
// Native builds never see these items, so the `extern "C"` trampolines
// emitted by `wit_bindgen::generate!()` never reach the native linker.

#[cfg(all(feature = "wit-bindings", target_family = "wasm"))]
mod wasm_impl {
    use anyhow::Result;
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::sync::atomic::{AtomicU64, Ordering};

    use tairitsu_vdom::{
        CanvasContext, DomRect, EventData, EventWitHandle, FocusEvent, GenericEvent, InputEvent,
        KeyboardEvent, MouseEvent, Platform, VNode,
    };

    use super::{WitElement, WitEvent, WitPlatform};

    unsafe extern "C" {
        fn tairitsu_component_bootstrap();
    }

    type EventCallback = Box<dyn FnMut(Box<dyn EventData>)>;
    type EventCallbackMap = HashMap<u64, EventCallback>;
    type TimeoutCallback = Option<Box<dyn FnOnce()>>;
    type IntervalCallback = Option<Box<dyn FnMut()>>;
    type AnimationCallback = Option<Box<dyn FnOnce(f64)>>;
    type ResizeObserverCallback = Box<dyn FnMut(Vec<tairitsu_vdom::ResizeObserverEntry>)>;
    type MutationObserverCallback = Box<dyn FnMut(Vec<tairitsu_vdom::MutationRecord>)>;

    static NEXT_CALLBACK_ID: AtomicU64 = AtomicU64::new(1);

    fn next_callback_id() -> u64 {
        NEXT_CALLBACK_ID.fetch_add(1, Ordering::SeqCst)
    }

    thread_local! {
        static EVENT_CALLBACKS: RefCell<EventCallbackMap> = RefCell::new(HashMap::new());
        static ELEMENT_LISTENERS: RefCell<HashMap<(u64, String), u64>>
            = RefCell::new(HashMap::new());
        static TIMEOUT_CALLBACKS: RefCell<HashMap<u64, TimeoutCallback>> = RefCell::new(HashMap::new());
        static INTERVAL_CALLBACKS: RefCell<HashMap<u64, IntervalCallback>> = RefCell::new(HashMap::new());
        static ANIMATION_CALLBACKS: RefCell<HashMap<u64, AnimationCallback>> = RefCell::new(HashMap::new());
        static RESIZE_OBSERVER_CALLBACKS: RefCell<HashMap<u64, ResizeObserverCallback>> = RefCell::new(HashMap::new());
        static MUTATION_OBSERVER_CALLBACKS: RefCell<HashMap<u64, MutationObserverCallback>> = RefCell::new(HashMap::new());
    }

    // ── WIT binding generation ───────────────────────────────────────────

    /// Rust bindings generated from `packages/browser-worlds/wit/browser-full.wit`.
    ///
    /// The `path` is relative to this crate's manifest directory
    /// (`packages/web/`).  `wit_bindgen::generate!` emits
    /// `cargo:rerun-if-changed` directives so the crate is rebuilt whenever
    /// the WIT files change.
    mod bindings {
        use super::BrowserComponent;

        wit_bindgen::generate!({
            path: "../browser-worlds/wit/browser-full.wit",
            world: "browser-full",
        });

        export!(BrowserComponent);
    }

    /// Set a style property on an element (implementation for static method).
    pub(super) fn set_style_static_impl(element: u64, name: &str, value: &str) -> Result<()> {
        bindings::tairitsu_browser::full::style::set_style_property(element, name, value)
            .map_err(|e| anyhow::anyhow!("set_style_property failed: {}", e))
    }

    // ── Component export implementation ─────────────────────────────────

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
            let event: Box<dyn EventData> = Box::new(
                MouseEvent::new()
                    .target(data.target)
                    .client_x(data.client_x as i32)
                    .client_y(data.client_y as i32)
                    .event_handle(wit_handle),
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

        fn on_generic_event(listener_id: u64, event_handle: u64, event_type: String) {
            let wit_handle = EventWitHandle::from_wit(event_handle);
            let event: Box<dyn EventData> = Box::new(
                GenericEvent::new()
                    .event_type(&event_type)
                    .event_handle(wit_handle),
            );
            dispatch_event(listener_id, &event_type, event);
        }
    }

    /// Dispatch an event to the registered callback with error handling.
    fn dispatch_event(listener_id: u64, event_type: &str, event: Box<dyn EventData>) {
        EVENT_CALLBACKS.with(|m| {
            let mut callbacks = m.borrow_mut();
            if let Some(handler) = callbacks.get_mut(&listener_id) {
                handler(event);
            } else {
                let msg = format!(
                    "Event dispatched but no callback registered: type={}, listener={}",
                    event_type, listener_id
                );
                log_warning(&msg);
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
                if let Some(Some(cb)) = callbacks.get_mut(&callback_id) {
                    cb();
                }
            });
        }
    }

    impl bindings::exports::tairitsu_browser::full::animation_callbacks::Guest for BrowserComponent {
        fn on_animation_frame(callback_id: u64, timestamp: f64) {
            ANIMATION_CALLBACKS.with(|m| {
                if let Some(callback) = m.borrow_mut().remove(&callback_id) {
                    if let Some(cb) = callback {
                        cb(timestamp);
                    }
                }
            });
        }
    }

    impl bindings::exports::tairitsu_browser::full::resize_observer_callbacks::Guest
        for BrowserComponent
    {
        fn on_resize(callback_id: u64, entries: Vec<u64>) {
            RESIZE_OBSERVER_CALLBACKS.with(|m| {
                let mut callbacks = m.borrow_mut();
                if let Some(handler) = callbacks.get_mut(&callback_id) {
                    let converted_entries: Vec<tairitsu_vdom::ResizeObserverEntry> = entries
                        .into_iter()
                        .map(|entry_handle| {
                            let target = bindings::tairitsu_browser::full::resize_observer_entry::get_target(entry_handle);
                            let content_rect = bindings::tairitsu_browser::full::resize_observer_entry::get_content_rect(entry_handle);

                            let border_box_handles = bindings::tairitsu_browser::full::resize_observer_entry::get_border_box_size(entry_handle);
                            let border_box_size: Vec<tairitsu_vdom::ResizeObserverSize> = border_box_handles
                                .into_iter()
                                .map(|size_handle| tairitsu_vdom::ResizeObserverSize {
                                    inline_size: bindings::tairitsu_browser::full::resize_observer_size::get_inline_size(size_handle),
                                    block_size: bindings::tairitsu_browser::full::resize_observer_size::get_block_size(size_handle),
                                })
                                .collect();

                            let content_box_handles = bindings::tairitsu_browser::full::resize_observer_entry::get_content_box_size(entry_handle);
                            let content_box_size: Vec<tairitsu_vdom::ResizeObserverSize> = content_box_handles
                                .into_iter()
                                .map(|size_handle| tairitsu_vdom::ResizeObserverSize {
                                    inline_size: bindings::tairitsu_browser::full::resize_observer_size::get_inline_size(size_handle),
                                    block_size: bindings::tairitsu_browser::full::resize_observer_size::get_block_size(size_handle),
                                })
                                .collect();

                            tairitsu_vdom::ResizeObserverEntry {
                                target,
                                content_rect: tairitsu_vdom::DomRect {
                                    x: content_rect.x,
                                    y: content_rect.y,
                                    width: content_rect.width,
                                    height: content_rect.height,
                                },
                                border_box_size,
                                content_box_size,
                            }
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
        fn on_mutation(callback_id: u64, records: Vec<u64>) {
            MUTATION_OBSERVER_CALLBACKS.with(|m| {
                let mut callbacks = m.borrow_mut();
                if let Some(handler) = callbacks.get_mut(&callback_id) {
                    let converted_records: Vec<tairitsu_vdom::MutationRecord> = records
                        .into_iter()
                        .map(|record_handle| {
                            tairitsu_vdom::MutationRecord {
                                record_type: bindings::tairitsu_browser::full::mutation_record::get_type(record_handle),
                                target: bindings::tairitsu_browser::full::mutation_record::get_target(record_handle),
                                added_nodes: vec![],
                                removed_nodes: vec![],
                                previous_sibling: bindings::tairitsu_browser::full::mutation_record::get_previous_sibling(record_handle),
                                next_sibling: bindings::tairitsu_browser::full::mutation_record::get_next_sibling(record_handle),
                                attribute_name: bindings::tairitsu_browser::full::mutation_record::get_attribute_name(record_handle),
                                attribute_namespace: bindings::tairitsu_browser::full::mutation_record::get_attribute_namespace(record_handle),
                                old_value: bindings::tairitsu_browser::full::mutation_record::get_old_value(record_handle),
                            }
                        })
                        .collect();
                    handler(converted_records);
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
                    bindings::tairitsu_browser::full::style::set_style_property(
                        element, property, value,
                    )
                },
                |element| {
                    let rect =
                        bindings::tairitsu_browser::full::platform_helpers::get_bounding_client_rect(
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
            // Try console operations
            bindings::tairitsu_browser::full::console::log(
                "[WitPlatform] Environment validation passed",
            );
        });

        match result {
            Ok(_) => Ok(()),
            Err(_) => Err("WIT host call failed during validation".to_string()),
        }
    }

    /// Log an error through the WIT console interface.
    fn log_error(message: &str) {
        let formatted = format!("[WitPlatform ERROR] {}", message);
        bindings::tairitsu_browser::full::console::error(&formatted);
    }

    /// Log a warning through the WIT console interface.
    fn log_warning(message: &str) {
        let formatted = format!("[WitPlatform WARNING] {}", message);
        bindings::tairitsu_browser::full::console::warn(&formatted);
    }

    /// Log diagnostic information through the WIT console interface.
    fn log_info(message: &str) {
        let formatted = format!("[WitPlatform] {}", message);
        bindings::tairitsu_browser::full::console::log(&formatted);
    }

    // ── Platform trait implementation ────────────────────────────────────

    impl Platform for WitPlatform {
        type Element = WitElement;
        type Event = WitEvent;

        fn create_element(&self, tag: &str) -> Self::Element {
            // New WIT: create-element returns u64 directly, takes optional options
            let handle = bindings::tairitsu_browser::full::document::create_element(tag, None);
            WitElement(handle)
        }

        fn create_text_node(&self, text: &str) -> Self::Element {
            // New WIT: create-text-node returns u64 directly
            let handle = bindings::tairitsu_browser::full::document::create_text_node(text);
            WitElement(handle)
        }

        fn append_child(&self, parent: &Self::Element, child: &Self::Element) {
            // New WIT: append-child returns u64 (the appended node)
            let _ = bindings::tairitsu_browser::full::node::append_child(parent.0, child.0);
        }

        fn remove_child(&self, parent: &Self::Element, child: &Self::Element) {
            // New WIT: remove-child returns u64 (the removed node)
            let _ = bindings::tairitsu_browser::full::node::remove_child(parent.0, child.0);
        }

        fn set_attribute(&self, element: &Self::Element, name: &str, value: &str) {
            // New WIT: set-attribute returns void
            bindings::tairitsu_browser::full::element::set_attribute(element.0, name, value);
        }

        fn remove_attribute(&self, element: &Self::Element, name: &str) {
            // New WIT: remove-attribute returns void
            bindings::tairitsu_browser::full::element::remove_attribute(element.0, name);
        }

        fn set_style(&self, element: &Self::Element, name: &str, value: &str) {
            // New WIT: set-style-property returns result<_, string>
            if let Err(e) =
                bindings::tairitsu_browser::full::style::set_style_property(element.0, name, value)
            {
                log_warning(&format!("set_style_property failed: {}", e));
            }
        }

        fn set_class(&self, element: &Self::Element, class: &str) {
            bindings::tairitsu_browser::full::element::set_attribute(element.0, "class", class);
        }

        fn add_event_listener(
            &self,
            element: &Self::Element,
            event: &str,
            handler: Box<dyn FnMut(Box<dyn EventData>)>,
        ) {
            // New WIT: add-event-listener returns result<u64, string>
            match bindings::tairitsu_browser::full::event_target::add_event_listener(
                element.0, event, false,
            ) {
                Ok(listener_id) => {
                    EVENT_CALLBACKS.with(|m| m.borrow_mut().insert(listener_id, handler));
                    ELEMENT_LISTENERS.with(|m| {
                        m.borrow_mut()
                            .insert((element.0, event.to_string()), listener_id);
                    });
                    log_info(&format!(
                        "Added event listener: event={}, listener={}",
                        event, listener_id
                    ));
                }
                Err(e) => {
                    log_error(&format!(
                        "add_event_listener({}, {}) failed: {}",
                        element.0, event, e
                    ));
                    panic!("WIT add-event-listener failed: {}", e);
                }
            }
        }

        fn remove_event_listener(&self, element: &Self::Element, event: &str) {
            let listener_id =
                ELEMENT_LISTENERS.with(|m| m.borrow_mut().remove(&(element.0, event.to_string())));

            if let Some(id) = listener_id {
                EVENT_CALLBACKS.with(|m| m.borrow_mut().remove(&id));
                if let Err(e) =
                    bindings::tairitsu_browser::full::event_target::remove_event_listener(
                        element.0, id,
                    )
                {
                    log_warning(&format!("remove_event_listener failed: {}", e));
                } else {
                    log_info(&format!(
                        "Removed event listener: event={}, listener={}",
                        event, id
                    ));
                }
            } else {
                log_warning(&format!(
                    "remove_event_listener: no listener found for event '{}' on element {}",
                    event, element.0
                ));
            }
        }

        fn get_bounding_client_rect(&self, element: &Self::Element) -> DomRect {
            let rect = bindings::tairitsu_browser::full::platform_helpers::get_bounding_client_rect(
                element.0,
            );
            DomRect {
                x: rect.x,
                y: rect.y,
                width: rect.width,
                height: rect.height,
            }
        }

        fn inner_width(&self) -> i32 {
            bindings::tairitsu_browser::full::platform_helpers::inner_width()
        }

        fn inner_height(&self) -> i32 {
            bindings::tairitsu_browser::full::platform_helpers::inner_height()
        }

        fn set_timeout(&self, callback: Box<dyn FnOnce()>, ms: i32) -> i32 {
            let callback_id = next_callback_id();
            TIMEOUT_CALLBACKS.with(|m| m.borrow_mut().insert(callback_id, Some(callback)));
            bindings::tairitsu_browser::full::platform_helpers::set_timeout(callback_id, ms)
        }

        fn clear_timeout(&self, id: i32) {
            bindings::tairitsu_browser::full::platform_helpers::clear_timeout(id)
        }

        fn request_animation_frame(&self, callback: Box<dyn FnOnce(f64)>) -> u32 {
            let callback_id = next_callback_id();
            ANIMATION_CALLBACKS.with(|m| m.borrow_mut().insert(callback_id, Some(callback)));
            bindings::tairitsu_browser::full::platform_helpers::request_animation_frame(callback_id)
        }

        fn cancel_animation_frame(&self, id: u32) {
            bindings::tairitsu_browser::full::platform_helpers::cancel_animation_frame(id)
        }

        fn get_canvas_context(
            &self,
            element: &Self::Element,
            context_type: &str,
        ) -> Option<CanvasContext> {
            bindings::tairitsu_browser::full::html_canvas_element::get_context(
                element.0,
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

        fn create_resize_observer(
            &self,
            callback: Box<dyn FnMut(Vec<tairitsu_vdom::ResizeObserverEntry>)>,
        ) -> u64 {
            let callback_id = next_callback_id();
            RESIZE_OBSERVER_CALLBACKS.with(|m| m.borrow_mut().insert(callback_id, callback));
            bindings::tairitsu_browser::full::platform_helpers::create_resize_observer(callback_id)
        }

        fn observe_resize(&self, observer: u64, element: &Self::Element) {
            bindings::tairitsu_browser::full::platform_helpers::observe_resize(observer, element.0);
        }

        fn unobserve_resize(&self, observer: u64, element: &Self::Element) {
            bindings::tairitsu_browser::full::platform_helpers::unobserve_resize(
                observer, element.0,
            );
        }

        fn disconnect_resize(&self, observer: u64) {
            bindings::tairitsu_browser::full::platform_helpers::disconnect_resize(observer);
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
            bindings::tairitsu_browser::full::platform_helpers::observe_mutations(
                observer, element.0, None,
            );
        }

        fn disconnect_mutation(&self, observer: u64) {
            bindings::tairitsu_browser::full::platform_helpers::disconnect_mutation(observer);
        }
    }

    pub(super) fn mount_vnode_to_app(platform: &WitPlatform, vnode: &VNode) -> Result<()> {
        // Document handle for non-element-parent-node operations (getElementById)
        // Document implements NonElementParentNode, so we use its handle
        let doc_handle: u64 = 0; // Global document singleton uses handle 0

        let app = if let Some(handle) =
            bindings::tairitsu_browser::full::non_element_parent_node::get_element_by_id(
                doc_handle, "app",
            ) {
            WitElement(handle)
        } else {
            let body = bindings::tairitsu_browser::full::document::get_body()
                .ok_or_else(|| anyhow::anyhow!("document.body is not available"))?;
            let div = bindings::tairitsu_browser::full::document::create_element("div", None);
            bindings::tairitsu_browser::full::element::set_attribute(div, "id", "app");
            let _ = bindings::tairitsu_browser::full::node::append_child(body, div);
            WitElement(div)
        };

        // set_text_content takes option<string>, returns void
        bindings::tairitsu_browser::full::node::set_text_content(app.0, Some(""));

        render_vnode(platform, vnode, &app)
    }

    fn render_vnode(platform: &WitPlatform, vnode: &VNode, parent: &WitElement) -> Result<()> {
        match vnode {
            VNode::Element(velement) => {
                let element = platform.create_element(&velement.tag);

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

                for child in &velement.children {
                    render_vnode(platform, child, &element)?;
                }

                platform.append_child(parent, &element);
            }
            VNode::Text(vtext) => {
                let text_node = platform.create_text_node(&vtext.text);
                platform.append_child(parent, &text_node);
            }
            VNode::Fragment(children) => {
                for child in children {
                    render_vnode(platform, child, parent)?;
                }
            }
        }

        Ok(())
    }
}
