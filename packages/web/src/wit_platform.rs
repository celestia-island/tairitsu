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
             For native or wasm32-unknown-unknown builds, use WebPlatform \
             with the `web` feature instead."
        );

        #[allow(unreachable_code)]
        Ok(Self)
    }

    /// Render a VNode tree into `#app` for WIT-backed browser components.
    ///
    /// This replaces the bootstrap text set by `lifecycle.start` and mounts
    /// the actual app view tree so users can see real UI content.
    pub fn mount_vnode_to_app(&self, vnode: &tairitsu_vdom::VNode) -> Result<()> {
        #[cfg(not(target_family = "wasm"))]
        anyhow::bail!(
            "mount_vnode_to_app is only available on wasm32 targets (wasm32-wasip2)"
        );

        #[cfg(target_family = "wasm")]
        {
            wasm_impl::mount_vnode_to_app(self, vnode)
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

    use tairitsu_vdom::{
        EventData, FocusEvent, InputEvent, KeyboardEvent, MouseEvent, Platform, VNode,
    };

    use super::{WitElement, WitEvent, WitPlatform};

    type EventCallback = Box<dyn FnMut(Box<dyn EventData>)>;
    type EventCallbackMap = HashMap<u64, EventCallback>;

    // ── Event dispatch tables ────────────────────────────────────────────

    thread_local! {
        /// Maps each WIT `listener-id` to the Rust event callback closure.
        /// Populated by [`WitPlatform::add_event_listener`]
        /// and cleared by [`WitPlatform::remove_event_listener`].
        static EVENT_CALLBACKS: RefCell<EventCallbackMap> = RefCell::new(HashMap::new());

        /// Maps `(node-handle, event-type-string)` → `listener-id`.
        /// Used by `remove_event_listener` to look up the id from
        /// the (element, event-name) pair the Platform trait provides.
        static ELEMENT_LISTENERS: RefCell<HashMap<(u64, String), u64>>
            = RefCell::new(HashMap::new());
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
            data: bindings::exports::tairitsu_browser::full::event_callbacks::MouseEventData,
        ) {
            let event: Box<dyn EventData> = Box::new(
                MouseEvent::new()
                    .client_x(data.client_x as i32)
                    .client_y(data.client_y as i32),
            );
            EVENT_CALLBACKS.with(|m| {
                if let Some(handler) = m.borrow_mut().get_mut(&listener_id) {
                    handler(event);
                }
            });
        }

        fn on_keyboard_event(
            listener_id: u64,
            data: bindings::exports::tairitsu_browser::full::event_callbacks::KeyboardEventData,
        ) {
            let event: Box<dyn EventData> = Box::new(KeyboardEvent::new().key(data.key));
            EVENT_CALLBACKS.with(|m| {
                if let Some(handler) = m.borrow_mut().get_mut(&listener_id) {
                    handler(event);
                }
            });
        }

        fn on_focus_event(
            listener_id: u64,
            _data: bindings::exports::tairitsu_browser::full::event_callbacks::FocusEventData,
        ) {
            let event: Box<dyn EventData> = Box::new(FocusEvent::new());
            EVENT_CALLBACKS.with(|m| {
                if let Some(handler) = m.borrow_mut().get_mut(&listener_id) {
                    handler(event);
                }
            });
        }

        fn on_input_event(
            listener_id: u64,
            data: bindings::exports::tairitsu_browser::full::event_callbacks::InputEventData,
        ) {
            let event: Box<dyn EventData> =
                Box::new(InputEvent::new().data(data.data.unwrap_or_default()));
            EVENT_CALLBACKS.with(|m| {
                if let Some(handler) = m.borrow_mut().get_mut(&listener_id) {
                    handler(event);
                }
            });
        }

        fn on_generic_event(listener_id: u64, _event_type: String) {
            let event: Box<dyn EventData> = Box::new(MouseEvent::new());
            EVENT_CALLBACKS.with(|m| {
                if let Some(handler) = m.borrow_mut().get_mut(&listener_id) {
                    handler(event);
                }
            });
        }
    }

    impl bindings::exports::tairitsu_browser::full::lifecycle::Guest for BrowserComponent {
        fn start() -> Result<(), String> {
            let app = if let Some(handle) = bindings::tairitsu_browser::full::document::get_element_by_id("app") {
                handle
            } else {
                let body = bindings::tairitsu_browser::full::document::body()
                    .ok_or_else(|| "document.body is not available".to_string())?;
                let div = bindings::tairitsu_browser::full::document::create_element("div")?;
                bindings::tairitsu_browser::full::node::set_attribute(div, "id", "app")?;
                bindings::tairitsu_browser::full::node::append_child(body, div)?;
                div
            };

            bindings::tairitsu_browser::full::node::set_text_content(
                app,
                "Tairitsu component started (lifecycle.start)"
            )?;

            Ok(())
        }
    }

    // ── Platform trait implementation ────────────────────────────────────

    impl Platform for WitPlatform {
        type Element = WitElement;
        type Event = WitEvent;

        fn create_element(&self, tag: &str) -> Self::Element {
            let handle = bindings::tairitsu_browser::full::document::create_element(tag)
                .expect("WIT create-element failed");
            WitElement(handle)
        }

        fn create_text_node(&self, text: &str) -> Self::Element {
            let handle = bindings::tairitsu_browser::full::document::create_text_node(text)
                .expect("WIT create-text-node failed");
            WitElement(handle)
        }

        fn append_child(&self, parent: &Self::Element, child: &Self::Element) {
            bindings::tairitsu_browser::full::node::append_child(parent.0, child.0)
                .expect("WIT append-child failed");
        }

        fn remove_child(&self, parent: &Self::Element, child: &Self::Element) {
            bindings::tairitsu_browser::full::node::remove_child(parent.0, child.0)
                .expect("WIT remove-child failed");
        }

        fn set_attribute(&self, element: &Self::Element, name: &str, value: &str) {
            bindings::tairitsu_browser::full::node::set_attribute(element.0, name, value)
                .expect("WIT set-attribute failed");
        }

        fn remove_attribute(&self, element: &Self::Element, name: &str) {
            bindings::tairitsu_browser::full::node::remove_attribute(element.0, name)
                .expect("WIT remove-attribute failed");
        }

        fn set_style(&self, element: &Self::Element, name: &str, value: &str) {
            bindings::tairitsu_browser::full::style::set_style_property(element.0, name, value)
                .expect("WIT set-style-property failed");
        }

        fn set_class(&self, element: &Self::Element, class: &str) {
            bindings::tairitsu_browser::full::node::set_attribute(element.0, "class", class)
                .expect("WIT set-class (set-attribute) failed");
        }

        fn add_event_listener(
            &self,
            element: &Self::Element,
            event: &str,
            handler: Box<dyn FnMut(Box<dyn EventData>)>,
        ) {
            let listener_id = bindings::tairitsu_browser::full::event_target::add_event_listener(
                element.0, event, false,
            )
            .expect("WIT add-event-listener failed");

            EVENT_CALLBACKS.with(|m| m.borrow_mut().insert(listener_id, handler));
            ELEMENT_LISTENERS.with(|m| {
                m.borrow_mut()
                    .insert((element.0, event.to_string()), listener_id);
            });
        }

        fn remove_event_listener(&self, element: &Self::Element, event: &str) {
            let listener_id =
                ELEMENT_LISTENERS.with(|m| m.borrow_mut().remove(&(element.0, event.to_string())));

            if let Some(id) = listener_id {
                EVENT_CALLBACKS.with(|m| m.borrow_mut().remove(&id));
                bindings::tairitsu_browser::full::event_target::remove_event_listener(
                    element.0, id,
                )
                .expect("WIT remove-event-listener failed");
            }
        }
    }

    pub(super) fn mount_vnode_to_app(platform: &WitPlatform, vnode: &VNode) -> Result<()> {
        let app = if let Some(handle) = bindings::tairitsu_browser::full::document::get_element_by_id("app") {
            WitElement(handle)
        } else {
            let body = bindings::tairitsu_browser::full::document::body()
                .ok_or_else(|| anyhow::anyhow!("document.body is not available"))?;
            let div = bindings::tairitsu_browser::full::document::create_element("div")
                .map_err(|e| anyhow::anyhow!("failed to create #app element: {e}"))?;
            bindings::tairitsu_browser::full::node::set_attribute(div, "id", "app")
                .map_err(|e| anyhow::anyhow!("failed to set #app id: {e}"))?;
            bindings::tairitsu_browser::full::node::append_child(body, div)
                .map_err(|e| anyhow::anyhow!("failed to append #app to body: {e}"))?;
            WitElement(div)
        };

        bindings::tairitsu_browser::full::node::set_text_content(app.0, "")
            .map_err(|e| anyhow::anyhow!("failed to clear #app content: {e}"))?;

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
