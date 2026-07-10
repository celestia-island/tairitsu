//! WIT linker registration for SSR
//!
//! This module registers all WIT interface implementations with the wasmtime Linker.

use anyhow::Result;

use wasmtime::component::{HasSelf, Linker};

use crate::{host_state::SsrHostState, stubs};

/// Register all SSR WIT implementations with the linker (direct version)
///
/// This version works directly with SsrHostState.
/// Auto-generated stubs are registered first, then manual implementations override them.
pub fn register_ssr_imports_direct(linker: &mut Linker<SsrHostState>) -> Result<()> {
    // Register bindgen-generated interfaces first (for proper type marshaling)
    // The SsrHostState implements the Host traits directly
    // We use HasSelf<SsrHostState> as the D parameter which implements HostWithStore
    crate::bindings::tairitsu_browser::full::resize_observer_entry::add_to_linker::<
        SsrHostState,
        HasSelf<SsrHostState>,
    >(linker, |state| -> &mut SsrHostState { state })?;

    crate::bindings::tairitsu_browser::full::resize_observer_size::add_to_linker::<
        SsrHostState,
        HasSelf<SsrHostState>,
    >(linker, |state| -> &mut SsrHostState { state })?;

    crate::bindings::tairitsu_browser::full::platform_helpers::add_to_linker::<
        SsrHostState,
        HasSelf<SsrHostState>,
    >(linker, |state| -> &mut SsrHostState { state })?;

    stubs::register_all_stubs(linker)?;
    register_core_imports(linker)?;
    Ok(())
}

/// Register core DOM imports that SSR actually needs
fn register_core_imports(linker: &mut Linker<SsrHostState>) -> Result<()> {
    // Note: Console interface removed - console operations now use direct browser console
    // via wasm-bindgen in the web package, not WIT interface

    // Document interface
    let mut document = linker.instance("tairitsu-browser:full/document@0.2.0")?;
    document.func_wrap(
        "create-element",
        |mut caller: wasmtime::StoreContextMut<'_, SsrHostState>,
         (local_name, _options): (String, Option<String>)|
         -> Result<(u64,), wasmtime::Error> {
            let state = caller.data_mut();
            Ok((state.dom.create_element(&local_name, None),))
        },
    )?;

    document.func_wrap(
        "create-text-node",
        |mut caller: wasmtime::StoreContextMut<'_, SsrHostState>,
         (data,): (String,)|
         -> Result<(u64,), wasmtime::Error> {
            let state = caller.data_mut();
            Ok((state.dom.create_text_node(&data),))
        },
    )?;

    document.func_wrap(
        "get-body",
        |mut caller: wasmtime::StoreContextMut<'_, SsrHostState>,
         (): ()|
         -> Result<(Option<u64>,), wasmtime::Error> {
            let state = caller.data_mut();
            Ok((Some(state.dom.body_handle()),))
        },
    )?;

    document.func_wrap(
        "get-head",
        |mut caller: wasmtime::StoreContextMut<'_, SsrHostState>,
         (): ()|
         -> Result<(Option<u64>,), wasmtime::Error> {
            let state = caller.data_mut();
            Ok((Some(state.dom.head_handle()),))
        },
    )?;

    document.func_wrap(
        "get-element-by-id",
        |caller: wasmtime::StoreContextMut<'_, SsrHostState>,
         (id,): (String,)|
         -> Result<(Option<u64>,), wasmtime::Error> {
            let dom = &caller.data().dom;
            Ok((dom.get_element_by_id(&id),))
        },
    )?;

    document.func_wrap(
        "query-selector",
        |caller: wasmtime::StoreContextMut<'_, SsrHostState>,
         (selector,): (String,)|
         -> Result<(Option<u64>,), wasmtime::Error> {
            let dom = &caller.data().dom;
            Ok((dom.query_selector(&selector),))
        },
    )?;

    // Node interface
    let mut node = linker.instance("tairitsu-browser:full/node@0.2.0")?;
    node.func_wrap(
        "append-child",
        |mut caller: wasmtime::StoreContextMut<'_, SsrHostState>,
         (parent, child): (u64, u64)|
         -> Result<(u64,), wasmtime::Error> {
            let state = caller.data_mut();
            state
                .dom
                .append_child(parent, child)
                .map_err(wasmtime::Error::msg)?;
            Ok((child,))
        },
    )?;

    node.func_wrap(
        "remove-child",
        |mut caller: wasmtime::StoreContextMut<'_, SsrHostState>,
         (parent, child): (u64, u64)|
         -> Result<(u64,), wasmtime::Error> {
            let state = caller.data_mut();
            state
                .dom
                .remove_child(parent, child)
                .map_err(wasmtime::Error::msg)?;
            Ok((child,))
        },
    )?;

    node.func_wrap(
        "set-attribute",
        |mut caller: wasmtime::StoreContextMut<'_, SsrHostState>,
         (handle, name, value): (u64, String, String)|
         -> Result<(), wasmtime::Error> {
            let state = caller.data_mut();
            if let Some(node) = state.dom.get_node_mut(handle) {
                node.set_attribute(&name, &value);
            }
            Ok(())
        },
    )?;

    node.func_wrap(
        "get-attribute",
        |caller: wasmtime::StoreContextMut<'_, SsrHostState>,
         (handle, name): (u64, String)|
         -> Result<(Option<String>,), wasmtime::Error> {
            let dom = &caller.data().dom;
            let value = dom
                .get_node(handle)
                .and_then(|n| n.get_attribute(&name).map(|s| s.to_string()));
            Ok((value,))
        },
    )?;

    node.func_wrap(
        "remove-attribute",
        |mut caller: wasmtime::StoreContextMut<'_, SsrHostState>,
         (handle, name): (u64, String)|
         -> Result<(), wasmtime::Error> {
            let state = caller.data_mut();
            if let Some(node) = state.dom.get_node_mut(handle) {
                node.remove_attribute(&name);
            }
            Ok(())
        },
    )?;

    node.func_wrap(
        "set-text-content",
        |mut caller: wasmtime::StoreContextMut<'_, SsrHostState>,
         (handle, text): (u64, Option<String>)|
         -> Result<(), wasmtime::Error> {
            let state = caller.data_mut();
            if let Some(text) = text {
                let _ = state.dom.set_text_content(handle, &text);
            } else {
                let _ = state.dom.set_text_content(handle, "");
            }
            Ok(())
        },
    )?;

    node.func_wrap(
        "get-text-content",
        |caller: wasmtime::StoreContextMut<'_, SsrHostState>,
         (handle,): (u64,)|
         -> Result<(Option<String>,), wasmtime::Error> {
            let dom = &caller.data().dom;
            let text = dom.get_text_content(handle);
            Ok((text,))
        },
    )?;

    // Element interface
    let mut element = linker.instance("tairitsu-browser:full/element@0.2.0")?;
    element.func_wrap(
        "set-attribute",
        |mut caller: wasmtime::StoreContextMut<'_, SsrHostState>,
         (self_handle, name, value): (u64, String, String)|
         -> Result<(), wasmtime::Error> {
            let state = caller.data_mut();
            if let Some(node) = state.dom.get_node_mut(self_handle) {
                node.set_attribute(&name, &value);
            }
            Ok(())
        },
    )?;

    element.func_wrap(
        "set-class-name",
        |mut caller: wasmtime::StoreContextMut<'_, SsrHostState>,
         (self_handle, value): (u64, String)| {
            let state = caller.data_mut();
            if let Some(node) = state.dom.get_node_mut(self_handle) {
                node.set_class(&value);
            }
            Ok(())
        },
    )?;

    element.func_wrap(
        "remove-attribute",
        |mut caller: wasmtime::StoreContextMut<'_, SsrHostState>,
         (self_handle, name): (u64, String)|
         -> Result<(), wasmtime::Error> {
            let state = caller.data_mut();
            if let Some(node) = state.dom.get_node_mut(self_handle) {
                node.remove_attribute(&name);
            }
            Ok(())
        },
    )?;

    // W3C CSSOM interfaces - ElementCSSInlineStyle and CSSStyleDeclaration

    // ElementCSSInlineStyle: get-style
    let mut element_css_inline_style =
        linker.instance("tairitsu-browser:css/element-css-inline-style@0.2.0")?;
    element_css_inline_style.func_wrap(
        "get-style",
        |_caller: wasmtime::StoreContextMut<'_, SsrHostState>,
         (element_handle,): (u64,)|
         -> Result<(u64,), wasmtime::Error> {
            // In SSR, we return the element handle itself as the style handle
            // The style declaration is stored as part of the element node
            Ok((element_handle,))
        },
    )?;

    // CSSStyleDeclaration: set-property
    let mut css_style_declaration =
        linker.instance("tairitsu-browser:css/css-style-declaration@0.2.0")?;
    css_style_declaration.func_wrap(
        "set-property",
        |mut caller: wasmtime::StoreContextMut<'_, SsrHostState>,
         (style_handle, property, value, _priority): (u64, String, String, Option<String>)|
         -> Result<(Result<(), String>,), wasmtime::Error> {
            let state = caller.data_mut();
            if let Some(node) = state.dom.get_node_mut(style_handle) {
                node.set_style_property(&property, &value);
                return Ok((Ok(()),));
            }
            Ok((Err("Element not found".to_string()),))
        },
    )?;

    // CSSStyleDeclaration: get-property-value
    css_style_declaration.func_wrap(
        "get-property-value",
        |caller: wasmtime::StoreContextMut<'_, SsrHostState>,
         (style_handle, property): (u64, String)|
         -> Result<(String,), wasmtime::Error> {
            let dom = &caller.data().dom;
            let value = dom
                .get_node(style_handle)
                .and_then(|n| n.get_style_property(&property))
                .unwrap_or_default()
                .to_string();
            Ok((value,))
        },
    )?;

    // CSSStyleDeclaration: remove-property
    css_style_declaration.func_wrap(
        "remove-property",
        |mut caller: wasmtime::StoreContextMut<'_, SsrHostState>,
         (style_handle, property): (u64, String)|
         -> Result<(String,), wasmtime::Error> {
            let state = caller.data_mut();
            let old_value = state
                .dom
                .get_node(style_handle)
                .and_then(|n| n.get_style_property(&property))
                .unwrap_or_default()
                .to_string();
            if let Some(node) = state.dom.get_node_mut(style_handle) {
                node.remove_style_property(&property);
            }
            Ok((old_value,))
        },
    )?;

    // Platform helpers interface - now using bindgen-generated Host trait
    // The implementation is in host_state.rs (PlatformHelpersHost trait)
    // NOTE: wit-bindgen may not generate set_interval/clear_interval in the
    // Host trait even though they're in the WIT. Register them manually as
    // no-op stubs so components that import them can instantiate.
    {
        let mut ph = linker.instance("tairitsu-browser:full/platform-helpers@0.2.0")?;
        // wit-bindgen doesn't generate these in the Host trait despite being in
        // the WIT. Register them manually as no-op stubs.
        ph.func_wrap(
            "set-timeout",
            |_caller: wasmtime::StoreContextMut<'_, SsrHostState>,
             (_callback_id, _ms): (u64, i32)|
             -> Result<(i32,), wasmtime::Error> { Ok((1,)) },
        )?;
        ph.func_wrap(
            "clear-timeout",
            |_caller: wasmtime::StoreContextMut<'_, SsrHostState>,
             (_id,): (i32,)|
             -> Result<(), wasmtime::Error> { Ok(()) },
        )?;
        ph.func_wrap(
            "request-animation-frame",
            |_caller: wasmtime::StoreContextMut<'_, SsrHostState>,
             (_callback_id,): (u64,)|
             -> Result<(u32,), wasmtime::Error> { Ok((1,)) },
        )?;
        ph.func_wrap(
            "cancel-animation-frame",
            |_caller: wasmtime::StoreContextMut<'_, SsrHostState>,
             (_id,): (u32,)|
             -> Result<(), wasmtime::Error> { Ok(()) },
        )?;
        ph.func_wrap(
            "set-interval",
            |_caller: wasmtime::StoreContextMut<'_, SsrHostState>,
             (_callback_id, _ms): (u64, i32)|
             -> Result<(i32,), wasmtime::Error> { Ok((1,)) },
        )?;
        ph.func_wrap(
            "clear-interval",
            |_caller: wasmtime::StoreContextMut<'_, SsrHostState>,
             (_id,): (i32,)|
             -> Result<(), wasmtime::Error> { Ok(()) },
        )?;
    }

    // Event target interface
    let mut event_target = linker.instance("tairitsu-browser:full/event-target@0.2.0")?;
    event_target.func_wrap(
        "add-event-listener",
        |_caller: wasmtime::StoreContextMut<'_, SsrHostState>,
         (_target, _event_type, _use_capture): (u64, String, bool)|
         -> Result<(Result<u64, String>,), wasmtime::Error> {
            // Return a dummy listener ID
            Ok((Ok(1),))
        },
    )?;

    event_target.func_wrap(
        "remove-event-listener",
        |_caller: wasmtime::StoreContextMut<'_, SsrHostState>,
         (_target, _listener_id): (u64, u64)|
         -> Result<(Result<(), String>,), wasmtime::Error> { Ok((Ok(()),)) },
    )?;

    event_target.func_wrap(
        "prevent-default",
        |_caller: wasmtime::StoreContextMut<'_, SsrHostState>,
         _event: (u64,)|
         -> Result<(), wasmtime::Error> { Ok(()) },
    )?;

    event_target.func_wrap(
        "stop-propagation",
        |_caller: wasmtime::StoreContextMut<'_, SsrHostState>,
         _event: (u64,)|
         -> Result<(), wasmtime::Error> { Ok(()) },
    )?;

    // Event methods
    let mut event = linker.instance("tairitsu-browser:full/event@0.2.0")?;
    event.func_wrap(
        "prevent-default",
        |_caller: wasmtime::StoreContextMut<'_, SsrHostState>,
         (_self,): (u64,)|
         -> Result<(), wasmtime::Error> { Ok(()) },
    )?;

    event.func_wrap(
        "stop-propagation",
        |_caller: wasmtime::StoreContextMut<'_, SsrHostState>,
         (_self,): (u64,)|
         -> Result<(), wasmtime::Error> { Ok(()) },
    )?;

    // Window
    let mut window = linker.instance("tairitsu-browser:full/window@0.2.0")?;
    window.func_wrap(
        "get-inner-width",
        |caller: wasmtime::StoreContextMut<'_, SsrHostState>,
         (): ()|
         -> Result<(i32,), wasmtime::Error> { Ok((caller.data().config.viewport_width,)) },
    )?;

    window.func_wrap(
        "get-inner-height",
        |caller: wasmtime::StoreContextMut<'_, SsrHostState>,
         (): ()|
         -> Result<(i32,), wasmtime::Error> { Ok((caller.data().config.viewport_height,)) },
    )?;

    // ── Stubs for interfaces the component uses but SSR doesn't fully implement.
    // These return no-op/default values so the component can mount its UI without
    // trapping on missing browser APIs.

    // non-element-parent-node: get-element-by-id
    {
        let mut nepn = linker.instance("tairitsu-browser:full/non-element-parent-node@0.2.0")?;
        nepn.func_wrap(
            "get-element-by-id",
            |caller: wasmtime::StoreContextMut<'_, SsrHostState>,
             (_self, element_id): (u64, String)|
             -> Result<(Option<u64>,), wasmtime::Error> {
                // Look up in the SSR DOM
                let handle = caller.data().dom.get_element_by_id(&element_id);
                Ok((handle,))
            },
        )?;
    }

    // parent-node: get-children, append, prepend, etc. (no-ops)
    {
        let mut pn = linker.instance("tairitsu-browser:full/parent-node@0.2.0")?;
        pn.func_wrap(
            "get-children",
            |_caller: wasmtime::StoreContextMut<'_, SsrHostState>, (_self,): (u64,)|
             -> Result<(u64,), wasmtime::Error> { Ok((0,)) },
        )?;
        pn.func_wrap(
            "get-first-element-child",
            |_caller: wasmtime::StoreContextMut<'_, SsrHostState>, (_self,): (u64,)|
             -> Result<(Option<u64>,), wasmtime::Error> { Ok((None,)) },
        )?;
        pn.func_wrap(
            "get-last-element-child",
            |_caller: wasmtime::StoreContextMut<'_, SsrHostState>, (_self,): (u64,)|
             -> Result<(Option<u64>,), wasmtime::Error> { Ok((None,)) },
        )?;
        pn.func_wrap(
            "get-child-element-count",
            |_caller: wasmtime::StoreContextMut<'_, SsrHostState>, (_self,): (u64,)|
             -> Result<(u32,), wasmtime::Error> { Ok((0,)) },
        )?;
        pn.func_wrap(
            "prepend",
            |_caller: wasmtime::StoreContextMut<'_, SsrHostState>, (_self, _nodes): (u64, Vec<String>)|
             -> Result<(), wasmtime::Error> { Ok(()) },
        )?;
        pn.func_wrap(
            "append",
            |_caller: wasmtime::StoreContextMut<'_, SsrHostState>, (_self, _nodes): (u64, Vec<String>)|
             -> Result<(), wasmtime::Error> { Ok(()) },
        )?;
    }

    // dom-token-list: add/remove/toggle/contains (no-ops)
    {
        let mut dtl = linker.instance("tairitsu-browser:full/dom-token-list@0.2.0")?;
        dtl.func_wrap(
            "add",
            |_caller: wasmtime::StoreContextMut<'_, SsrHostState>, (_self, _tokens): (u64, Vec<String>)|
             -> Result<(), wasmtime::Error> { Ok(()) },
        )?;
        dtl.func_wrap(
            "remove",
            |_caller: wasmtime::StoreContextMut<'_, SsrHostState>, (_self, _tokens): (u64, Vec<String>)|
             -> Result<(), wasmtime::Error> { Ok(()) },
        )?;
        dtl.func_wrap(
            "contains",
            |_caller: wasmtime::StoreContextMut<'_, SsrHostState>, (_self, _token): (u64, String)|
             -> Result<(bool,), wasmtime::Error> { Ok((false,)) },
        )?;
        dtl.func_wrap(
            "toggle",
            |_caller: wasmtime::StoreContextMut<'_, SsrHostState>, (_self, _token, _force): (u64, String, Option<bool>)|
             -> Result<(bool,), wasmtime::Error> { Ok((false,)) },
        )?;
    }

    // node-list: get-length, item (no-op)
    {
        let mut nl = linker.instance("tairitsu-browser:full/node-list@0.2.0")?;
        nl.func_wrap(
            "get-length",
            |_caller: wasmtime::StoreContextMut<'_, SsrHostState>, (_self,): (u64,)|
             -> Result<(u32,), wasmtime::Error> { Ok((0,)) },
        )?;
        nl.func_wrap(
            "item",
            |_caller: wasmtime::StoreContextMut<'_, SsrHostState>, (_self, _index): (u64, u32)|
             -> Result<(Option<u64>,), wasmtime::Error> { Ok((None,)) },
        )?;
    }

    // history: back/forward/push-state (no-ops)
    {
        let mut hist = linker.instance("tairitsu-browser:full/history@0.2.0")?;
        hist.func_wrap(
            "back",
            |_caller: wasmtime::StoreContextMut<'_, SsrHostState>, (_self,): (u64,)|
             -> Result<(), wasmtime::Error> { Ok(()) },
        )?;
        hist.func_wrap(
            "forward",
            |_caller: wasmtime::StoreContextMut<'_, SsrHostState>, (_self,): (u64,)|
             -> Result<(), wasmtime::Error> { Ok(()) },
        )?;
        hist.func_wrap(
            "push-state",
            |_caller: wasmtime::StoreContextMut<'_, SsrHostState>, (_self, _data, _title, _url): (u64, String, String, Option<String>)|
             -> Result<(), wasmtime::Error> { Ok(()) },
        )?;
    }

    // location: href, pathname, etc.
    {
        let mut loc = linker.instance("tairitsu-browser:full/location@0.2.0")?;
        loc.func_wrap(
            "get-href",
            |_caller: wasmtime::StoreContextMut<'_, SsrHostState>, (_self,): (u64,)|
             -> Result<(String,), wasmtime::Error> { Ok(("/".to_string(),)) },
        )?;
        loc.func_wrap(
            "get-pathname",
            |_caller: wasmtime::StoreContextMut<'_, SsrHostState>, (_self,): (u64,)|
             -> Result<(String,), wasmtime::Error> { Ok(("/".to_string(),)) },
        )?;
    }

    Ok(())
}
