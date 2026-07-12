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

    // Skip bindgen platform_helpers::add_to_linker — it registers functions
    // with types that don't match the component's WIT version (e.g.
    // set-timeout, request-animation-frame). Manual stubs in register_core_imports
    // provide the correct signatures.
    // crate::bindings::tairitsu_browser::full::platform_helpers::add_to_linker::<
    //     SsrHostState,
    //     HasSelf<SsrHostState>,
    // >(linker, |state| -> &mut SsrHostState { state })?;

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

    // Additional node stubs for functions bindgen doesn't generate correctly
    node.func_wrap(
        "get-child-nodes",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         (_self,): (u64,)|
         -> Result<(u64,), wasmtime::Error> { Ok((0,)) },
    )?;
    node.func_wrap(
        "get-first-child",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         (_self,): (u64,)|
         -> Result<(Option<u64>,), wasmtime::Error> { Ok((None,)) },
    )?;
    node.func_wrap(
        "get-last-child",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         (_self,): (u64,)|
         -> Result<(Option<u64>,), wasmtime::Error> { Ok((None,)) },
    )?;
    node.func_wrap(
        "get-parent-node",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         (_self,): (u64,)|
         -> Result<(Option<u64>,), wasmtime::Error> { Ok((None,)) },
    )?;
    node.func_wrap(
        "get-parent-element",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         (_self,): (u64,)|
         -> Result<(Option<u64>,), wasmtime::Error> { Ok((None,)) },
    )?;
    node.func_wrap(
        "get-previous-sibling",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         (_self,): (u64,)|
         -> Result<(Option<u64>,), wasmtime::Error> { Ok((None,)) },
    )?;
    node.func_wrap(
        "get-next-sibling",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         (_self,): (u64,)|
         -> Result<(Option<u64>,), wasmtime::Error> { Ok((None,)) },
    )?;
    node.func_wrap(
        "has-child-nodes",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         (_self,): (u64,)|
         -> Result<(bool,), wasmtime::Error> { Ok((false,)) },
    )?;
    node.func_wrap(
        "insert-before",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         (_self, _node, _child): (u64, u64, Option<u64>)|
         -> Result<(u64,), wasmtime::Error> { Ok((0,)) },
    )?;
    node.func_wrap(
        "replace-child",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         (_self, _node, _child): (u64, u64, u64)|
         -> Result<(u64,), wasmtime::Error> { Ok((0,)) },
    )?;
    node.func_wrap(
        "clone-node",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         (_self, _subtree): (u64, Option<bool>)|
         -> Result<(u64,), wasmtime::Error> { Ok((0,)) },
    )?;
    node.func_wrap(
        "normalize",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         (_self,): (u64,)|
         -> Result<(), wasmtime::Error> { Ok(()) },
    )?;
    node.func_wrap(
        "get-node-name",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         (_self,): (u64,)|
         -> Result<(String,), wasmtime::Error> { Ok((String::new(),)) },
    )?;
    node.func_wrap(
        "get-node-type",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         (_self,): (u64,)|
         -> Result<(u16,), wasmtime::Error> { Ok((0,)) },
    )?;
    node.func_wrap(
        "get-owner-document",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         (_self,): (u64,)|
         -> Result<(Option<u64>,), wasmtime::Error> { Ok((None,)) },
    )?;
    node.func_wrap(
        "get-base-uri",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         (_self,): (u64,)|
         -> Result<(String,), wasmtime::Error> { Ok((String::new(),)) },
    )?;
    node.func_wrap(
        "get-is-connected",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         (_self,): (u64,)|
         -> Result<(bool,), wasmtime::Error> { Ok((false,)) },
    )?;
    node.func_wrap(
        "get-root-node",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         (_self, _options): (u64, Option<u64>)|
         -> Result<(u64,), wasmtime::Error> { Ok((0,)) },
    )?;
    node.func_wrap(
        "get-node-value",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         (_self,): (u64,)|
         -> Result<(Option<String>,), wasmtime::Error> { Ok((None,)) },
    )?;
    node.func_wrap(
        "set-node-value",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         (_self, _value): (u64, Option<String>)|
         -> Result<(), wasmtime::Error> { Ok(()) },
    )?;
    node.func_wrap(
        "is-equal-node",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         (_self, _other): (u64, Option<u64>)|
         -> Result<(bool,), wasmtime::Error> { Ok((false,)) },
    )?;
    node.func_wrap(
        "is-same-node",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         (_self, _other): (u64, Option<u64>)|
         -> Result<(bool,), wasmtime::Error> { Ok((false,)) },
    )?;
    node.func_wrap(
        "contains",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         (_self, _other): (u64, Option<u64>)|
         -> Result<(bool,), wasmtime::Error> { Ok((false,)) },
    )?;
    node.func_wrap(
        "lookup-prefix",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         (_self, _ns): (u64, Option<String>)|
         -> Result<(Option<String>,), wasmtime::Error> { Ok((None,)) },
    )?;
    node.func_wrap(
        "lookup-namespace-uri",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         (_self, _prefix): (u64, Option<String>)|
         -> Result<(Option<String>,), wasmtime::Error> { Ok((None,)) },
    )?;
    node.func_wrap(
        "is-default-namespace",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         (_self, _ns): (u64, Option<String>)|
         -> Result<(bool,), wasmtime::Error> { Ok((false,)) },
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

    // Auto-generated element stubs from WIT (all element interface functions)
    element.func_wrap(
        "get-client-rects",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         _: (u64,)|
         -> Result<(u64,), wasmtime::Error> { Ok((0,)) },
    )?;
    element.func_wrap(
        "get-bounding-client-rect",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         _: (u64,)|
         -> Result<(crate::bindings::DomRect,), wasmtime::Error> {
            Ok((crate::bindings::DomRect {
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 0.0,
            },))
        },
    )?;
    element.func_wrap(
        "check-visibility",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         _: (u64, Option<u64>)|
         -> Result<(bool,), wasmtime::Error> { Ok((false,)) },
    )?;
    element.func_wrap(
        "scroll-into-view",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         _: (u64, Option<bool>)|
         -> Result<(u64,), wasmtime::Error> { Ok((0,)) },
    )?;
    element.func_wrap(
        "scroll",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         _: (u64, Option<u64>)|
         -> Result<(u64,), wasmtime::Error> { Ok((0,)) },
    )?;
    element.func_wrap(
        "scroll-to",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         _: (u64, Option<u64>)|
         -> Result<(u64,), wasmtime::Error> { Ok((0,)) },
    )?;
    element.func_wrap(
        "scroll-by",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         _: (u64, Option<u64>)|
         -> Result<(u64,), wasmtime::Error> { Ok((0,)) },
    )?;
    element.func_wrap(
        "get-scroll-top",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         _: (u64,)|
         -> Result<(f64,), wasmtime::Error> { Ok((0.0,)) },
    )?;
    element.func_wrap(
        "set-scroll-top",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         _: (u64, f64)|
         -> Result<(), wasmtime::Error> { Ok(()) },
    )?;
    element.func_wrap(
        "get-scroll-left",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         _: (u64,)|
         -> Result<(f64,), wasmtime::Error> { Ok((0.0,)) },
    )?;
    element.func_wrap(
        "set-scroll-left",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         _: (u64, f64)|
         -> Result<(), wasmtime::Error> { Ok(()) },
    )?;
    element.func_wrap(
        "get-scroll-width",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         _: (u64,)|
         -> Result<(i32,), wasmtime::Error> { Ok((0,)) },
    )?;
    element.func_wrap(
        "get-scroll-height",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         _: (u64,)|
         -> Result<(i32,), wasmtime::Error> { Ok((0,)) },
    )?;
    element.func_wrap(
        "get-client-top",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         _: (u64,)|
         -> Result<(i32,), wasmtime::Error> { Ok((0,)) },
    )?;
    element.func_wrap(
        "get-client-left",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         _: (u64,)|
         -> Result<(i32,), wasmtime::Error> { Ok((0,)) },
    )?;
    element.func_wrap(
        "get-client-width",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         _: (u64,)|
         -> Result<(i32,), wasmtime::Error> { Ok((0,)) },
    )?;
    element.func_wrap(
        "get-client-height",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         _: (u64,)|
         -> Result<(i32,), wasmtime::Error> { Ok((0,)) },
    )?;
    element.func_wrap(
        "get-current-css-zoom",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         _: (u64,)|
         -> Result<(f64,), wasmtime::Error> { Ok((0.0,)) },
    )?;
    element.func_wrap(
        "get-namespace-uri",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         _: (u64,)|
         -> Result<(Option<String>,), wasmtime::Error> { Ok((None,)) },
    )?;
    element.func_wrap(
        "get-prefix",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         _: (u64,)|
         -> Result<(Option<String>,), wasmtime::Error> { Ok((None,)) },
    )?;
    element.func_wrap(
        "get-local-name",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         _: (u64,)|
         -> Result<(String,), wasmtime::Error> { Ok((String::new(),)) },
    )?;
    element.func_wrap(
        "get-tag-name",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         _: (u64,)|
         -> Result<(String,), wasmtime::Error> { Ok((String::new(),)) },
    )?;
    element.func_wrap(
        "get-id",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         _: (u64,)|
         -> Result<(String,), wasmtime::Error> { Ok((String::new(),)) },
    )?;
    element.func_wrap(
        "set-id",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         _: (u64, String)|
         -> Result<(), wasmtime::Error> { Ok(()) },
    )?;
    element.func_wrap(
        "get-class-name",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         _: (u64,)|
         -> Result<(String,), wasmtime::Error> { Ok((String::new(),)) },
    )?;
    element.func_wrap(
        "set-class-name",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         _: (u64, String)|
         -> Result<(), wasmtime::Error> { Ok(()) },
    )?;
    element.func_wrap(
        "get-class-list",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         _: (u64,)|
         -> Result<(u64,), wasmtime::Error> { Ok((0,)) },
    )?;
    element.func_wrap(
        "get-slot",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         _: (u64,)|
         -> Result<(String,), wasmtime::Error> { Ok((String::new(),)) },
    )?;
    element.func_wrap(
        "set-slot",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         _: (u64, String)|
         -> Result<(), wasmtime::Error> { Ok(()) },
    )?;
    element.func_wrap(
        "has-attributes",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         _: (u64,)|
         -> Result<(bool,), wasmtime::Error> { Ok((false,)) },
    )?;
    element.func_wrap(
        "get-attributes",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         _: (u64,)|
         -> Result<(u64,), wasmtime::Error> { Ok((0,)) },
    )?;
    element.func_wrap(
        "get-attribute-names",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         _: (u64,)|
         -> Result<(Vec<String>,), wasmtime::Error> { Ok((vec![],)) },
    )?;
    element.func_wrap(
        "get-attribute",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         _: (u64, String)|
         -> Result<(Option<String>,), wasmtime::Error> { Ok((None,)) },
    )?;
    element.func_wrap(
        "get-attribute-ns",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         _: (u64, Option<String>, String)|
         -> Result<(Option<String>,), wasmtime::Error> { Ok((None,)) },
    )?;
    element.func_wrap(
        "set-attribute",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         _: (u64, String, String)|
         -> Result<(), wasmtime::Error> { Ok(()) },
    )?;
    element.func_wrap(
        "set-attribute-ns",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         _: (u64, Option<String>, String, String)|
         -> Result<(), wasmtime::Error> { Ok(()) },
    )?;
    element.func_wrap(
        "remove-attribute",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         _: (u64, String)|
         -> Result<(), wasmtime::Error> { Ok(()) },
    )?;
    element.func_wrap(
        "remove-attribute-ns",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         _: (u64, Option<String>, String)|
         -> Result<(), wasmtime::Error> { Ok(()) },
    )?;
    element.func_wrap(
        "toggle-attribute",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         _: (u64, String, Option<bool>)|
         -> Result<(bool,), wasmtime::Error> { Ok((false,)) },
    )?;
    element.func_wrap(
        "has-attribute",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         _: (u64, String)|
         -> Result<(bool,), wasmtime::Error> { Ok((false,)) },
    )?;
    element.func_wrap(
        "has-attribute-ns",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         _: (u64, Option<String>, String)|
         -> Result<(bool,), wasmtime::Error> { Ok((false,)) },
    )?;
    element.func_wrap(
        "get-attribute-node",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         _: (u64, String)|
         -> Result<(Option<u64>,), wasmtime::Error> { Ok((None,)) },
    )?;
    element.func_wrap(
        "get-attribute-node-ns",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         _: (u64, Option<String>, String)|
         -> Result<(Option<u64>,), wasmtime::Error> { Ok((None,)) },
    )?;
    element.func_wrap(
        "set-attribute-node",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         _: (u64, u64)|
         -> Result<(Option<u64>,), wasmtime::Error> { Ok((None,)) },
    )?;
    element.func_wrap(
        "set-attribute-node-ns",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         _: (u64, u64)|
         -> Result<(Option<u64>,), wasmtime::Error> { Ok((None,)) },
    )?;
    element.func_wrap(
        "remove-attribute-node",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         _: (u64, u64)|
         -> Result<(u64,), wasmtime::Error> { Ok((0,)) },
    )?;
    element.func_wrap(
        "attach-shadow",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         _: (u64, u64)|
         -> Result<(u64,), wasmtime::Error> { Ok((0,)) },
    )?;
    element.func_wrap(
        "get-shadow-root",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         _: (u64,)|
         -> Result<(Option<u64>,), wasmtime::Error> { Ok((None,)) },
    )?;
    element.func_wrap(
        "get-custom-element-registry",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         _: (u64,)|
         -> Result<(Option<u64>,), wasmtime::Error> { Ok((None,)) },
    )?;
    element.func_wrap(
        "closest",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         _: (u64, String)|
         -> Result<(Option<u64>,), wasmtime::Error> { Ok((None,)) },
    )?;
    element.func_wrap(
        "matches",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         _: (u64, String)|
         -> Result<(bool,), wasmtime::Error> { Ok((false,)) },
    )?;
    element.func_wrap(
        "webkit-matches-selector",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         _: (u64, String)|
         -> Result<(bool,), wasmtime::Error> { Ok((false,)) },
    )?;
    element.func_wrap(
        "get-elements-by-tag-name",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         _: (u64, String)|
         -> Result<(u64,), wasmtime::Error> { Ok((0,)) },
    )?;
    element.func_wrap(
        "get-elements-by-tag-name-ns",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         _: (u64, Option<String>, String)|
         -> Result<(u64,), wasmtime::Error> { Ok((0,)) },
    )?;
    element.func_wrap(
        "get-elements-by-class-name",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         _: (u64, String)|
         -> Result<(u64,), wasmtime::Error> { Ok((0,)) },
    )?;
    element.func_wrap(
        "insert-adjacent-element",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         _: (u64, String, u64)|
         -> Result<(Option<u64>,), wasmtime::Error> { Ok((None,)) },
    )?;
    element.func_wrap(
        "insert-adjacent-text",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         _: (u64, String, String)|
         -> Result<(), wasmtime::Error> { Ok(()) },
    )?;
    element.func_wrap(
        "request-fullscreen",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         _: (u64, Option<u64>)|
         -> Result<(u64,), wasmtime::Error> { Ok((0,)) },
    )?;
    element.func_wrap(
        "get-onfullscreenchange",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         _: (u64,)|
         -> Result<(u64,), wasmtime::Error> { Ok((0,)) },
    )?;
    element.func_wrap(
        "set-onfullscreenchange",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         _: (u64, u64)|
         -> Result<(), wasmtime::Error> { Ok(()) },
    )?;
    element.func_wrap(
        "get-onfullscreenerror",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         _: (u64,)|
         -> Result<(u64,), wasmtime::Error> { Ok((0,)) },
    )?;
    element.func_wrap(
        "set-onfullscreenerror",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         _: (u64, u64)|
         -> Result<(), wasmtime::Error> { Ok(()) },
    )?;
    element.func_wrap(
        "set-html",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         _: (u64, String, Option<u64>)|
         -> Result<(), wasmtime::Error> { Ok(()) },
    )?;
    element.func_wrap(
        "set-html-unsafe",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         _: (u64, String, Option<u64>)|
         -> Result<(), wasmtime::Error> { Ok(()) },
    )?;
    element.func_wrap(
        "get-html",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         _: (u64, Option<u64>)|
         -> Result<(String,), wasmtime::Error> { Ok((String::new(),)) },
    )?;
    element.func_wrap(
        "get-inner-html",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         _: (u64,)|
         -> Result<(String,), wasmtime::Error> { Ok((String::new(),)) },
    )?;
    element.func_wrap(
        "set-inner-html",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         _: (u64, String)|
         -> Result<(), wasmtime::Error> { Ok(()) },
    )?;
    element.func_wrap(
        "get-outer-html",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         _: (u64,)|
         -> Result<(String,), wasmtime::Error> { Ok((String::new(),)) },
    )?;
    element.func_wrap(
        "set-outer-html",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         _: (u64, String)|
         -> Result<(), wasmtime::Error> { Ok(()) },
    )?;
    element.func_wrap(
        "insert-adjacent-html",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         _: (u64, String, String)|
         -> Result<(), wasmtime::Error> { Ok(()) },
    )?;
    element.func_wrap(
        "set-pointer-capture",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         _: (u64, i32)|
         -> Result<(), wasmtime::Error> { Ok(()) },
    )?;
    element.func_wrap(
        "release-pointer-capture",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         _: (u64, i32)|
         -> Result<(), wasmtime::Error> { Ok(()) },
    )?;
    element.func_wrap(
        "has-pointer-capture",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         _: (u64, i32)|
         -> Result<(bool,), wasmtime::Error> { Ok((false,)) },
    )?;

    // W3C CSSOM interfaces - ElementCSSInlineStyle and CSSStyleDeclaration

    // ElementCSSInlineStyle: get-style
    let mut element_css_inline_style =
        linker.instance("tairitsu-browser:full/element-css-inline-style@0.2.0")?;
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
        linker.instance("tairitsu-browser:full/css-style-declaration@0.2.0")?;
    css_style_declaration.func_wrap(
        "set-property",
        |mut caller: wasmtime::StoreContextMut<'_, SsrHostState>,
         (style_handle, property, value, _priority): (u64, String, String, Option<String>)|
         -> Result<(), wasmtime::Error> {
            let state = caller.data_mut();
            if let Some(node) = state.dom.get_node_mut(style_handle) {
                node.set_style_property(&property, &value);
            }
            Ok(())
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

    // Auto-generated event stubs from WIT
    let mut event = linker.instance("tairitsu-browser:full/event@0.2.0")?;
    event.func_wrap(
        "get-type",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         _: (u64,)|
         -> Result<(String,), wasmtime::Error> { Ok((String::new(),)) },
    )?;
    event.func_wrap(
        "get-target",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         _: (u64,)|
         -> Result<(Option<u64>,), wasmtime::Error> { Ok((None,)) },
    )?;
    event.func_wrap(
        "get-src-element",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         _: (u64,)|
         -> Result<(Option<u64>,), wasmtime::Error> { Ok((None,)) },
    )?;
    event.func_wrap(
        "get-current-target",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         _: (u64,)|
         -> Result<(Option<u64>,), wasmtime::Error> { Ok((None,)) },
    )?;
    event.func_wrap(
        "composed-path",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         _: (u64,)|
         -> Result<(Vec<u64>,), wasmtime::Error> { Ok((vec![],)) },
    )?;
    event.func_wrap(
        "get-event-phase",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         _: (u64,)|
         -> Result<(u16,), wasmtime::Error> { Ok((0,)) },
    )?;
    event.func_wrap(
        "stop-propagation",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         _: (u64,)|
         -> Result<(), wasmtime::Error> { Ok(()) },
    )?;
    event.func_wrap(
        "get-cancel-bubble",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         _: (u64,)|
         -> Result<(bool,), wasmtime::Error> { Ok((false,)) },
    )?;
    event.func_wrap(
        "set-cancel-bubble",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         _: (u64, bool)|
         -> Result<(), wasmtime::Error> { Ok(()) },
    )?;
    event.func_wrap(
        "stop-immediate-propagation",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         _: (u64,)|
         -> Result<(), wasmtime::Error> { Ok(()) },
    )?;
    event.func_wrap(
        "get-bubbles",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         _: (u64,)|
         -> Result<(bool,), wasmtime::Error> { Ok((false,)) },
    )?;
    event.func_wrap(
        "get-cancelable",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         _: (u64,)|
         -> Result<(bool,), wasmtime::Error> { Ok((false,)) },
    )?;
    event.func_wrap(
        "get-return-value",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         _: (u64,)|
         -> Result<(bool,), wasmtime::Error> { Ok((false,)) },
    )?;
    event.func_wrap(
        "set-return-value",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         _: (u64, bool)|
         -> Result<(), wasmtime::Error> { Ok(()) },
    )?;
    event.func_wrap(
        "prevent-default",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         _: (u64,)|
         -> Result<(), wasmtime::Error> { Ok(()) },
    )?;
    event.func_wrap(
        "get-default-prevented",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         _: (u64,)|
         -> Result<(bool,), wasmtime::Error> { Ok((false,)) },
    )?;
    event.func_wrap(
        "get-composed",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         _: (u64,)|
         -> Result<(bool,), wasmtime::Error> { Ok((false,)) },
    )?;
    event.func_wrap(
        "get-is-trusted",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         _: (u64,)|
         -> Result<(bool,), wasmtime::Error> { Ok((false,)) },
    )?;
    event.func_wrap(
        "get-time-stamp",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         _: (u64,)|
         -> Result<(f64,), wasmtime::Error> { Ok((0.0,)) },
    )?;
    event.func_wrap(
        "init-event",
        |_: wasmtime::StoreContextMut<'_, SsrHostState>,
         _: (u64, String, Option<bool>, Option<bool>)|
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

    // Additional window methods that bindgen doesn't generate correctly
    window.func_wrap(
        "get-computed-style",
        |_caller: wasmtime::StoreContextMut<'_, SsrHostState>,
         (_elt, _pseudo): (u64, Option<String>)|
         -> Result<(u64,), wasmtime::Error> { Ok((0,)) },
    )?;
    window.func_wrap(
        "get-device-pixel-ratio",
        |_caller: wasmtime::StoreContextMut<'_, SsrHostState>,
         (): ()|
         -> Result<(f64,), wasmtime::Error> { Ok((1.0,)) },
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
            |_caller: wasmtime::StoreContextMut<'_, SsrHostState>,
             (_self,): (u64,)|
             -> Result<(u64,), wasmtime::Error> { Ok((0,)) },
        )?;
        pn.func_wrap(
            "get-first-element-child",
            |_caller: wasmtime::StoreContextMut<'_, SsrHostState>,
             (_self,): (u64,)|
             -> Result<(Option<u64>,), wasmtime::Error> { Ok((None,)) },
        )?;
        pn.func_wrap(
            "get-last-element-child",
            |_caller: wasmtime::StoreContextMut<'_, SsrHostState>,
             (_self,): (u64,)|
             -> Result<(Option<u64>,), wasmtime::Error> { Ok((None,)) },
        )?;
        pn.func_wrap(
            "get-child-element-count",
            |_caller: wasmtime::StoreContextMut<'_, SsrHostState>,
             (_self,): (u64,)|
             -> Result<(u32,), wasmtime::Error> { Ok((0,)) },
        )?;
        pn.func_wrap(
            "query-selector",
            |_: wasmtime::StoreContextMut<'_, SsrHostState>,
             (_self, _s): (u64, String)|
             -> Result<(Option<u64>,), wasmtime::Error> { Ok((None,)) },
        )?;
        pn.func_wrap(
            "query-selector-all",
            |_: wasmtime::StoreContextMut<'_, SsrHostState>,
             (_self, _s): (u64, String)|
             -> Result<(u64,), wasmtime::Error> { Ok((0,)) },
        )?;
        pn.func_wrap(
            "prepend",
            |_caller: wasmtime::StoreContextMut<'_, SsrHostState>,
             (_self, _nodes): (u64, Vec<String>)|
             -> Result<(), wasmtime::Error> { Ok(()) },
        )?;
        pn.func_wrap(
            "append",
            |_caller: wasmtime::StoreContextMut<'_, SsrHostState>,
             (_self, _nodes): (u64, Vec<String>)|
             -> Result<(), wasmtime::Error> { Ok(()) },
        )?;
    }

    // dom-token-list: add/remove/toggle/contains (no-ops)
    {
        let mut dtl = linker.instance("tairitsu-browser:full/dom-token-list@0.2.0")?;
        dtl.func_wrap(
            "add",
            |_caller: wasmtime::StoreContextMut<'_, SsrHostState>,
             (_self, _tokens): (u64, Vec<String>)|
             -> Result<(), wasmtime::Error> { Ok(()) },
        )?;
        dtl.func_wrap(
            "remove",
            |_caller: wasmtime::StoreContextMut<'_, SsrHostState>,
             (_self, _tokens): (u64, Vec<String>)|
             -> Result<(), wasmtime::Error> { Ok(()) },
        )?;
        dtl.func_wrap(
            "contains",
            |_caller: wasmtime::StoreContextMut<'_, SsrHostState>,
             (_self, _token): (u64, String)|
             -> Result<(bool,), wasmtime::Error> { Ok((false,)) },
        )?;
        dtl.func_wrap(
            "toggle",
            |_caller: wasmtime::StoreContextMut<'_, SsrHostState>,
             (_self, _token, _force): (u64, String, Option<bool>)|
             -> Result<(bool,), wasmtime::Error> { Ok((false,)) },
        )?;
    }

    // node-list: get-length, item (no-op)
    {
        let mut nl = linker.instance("tairitsu-browser:full/node-list@0.2.0")?;
        nl.func_wrap(
            "get-length",
            |_caller: wasmtime::StoreContextMut<'_, SsrHostState>,
             (_self,): (u64,)|
             -> Result<(u32,), wasmtime::Error> { Ok((0,)) },
        )?;
        nl.func_wrap(
            "item",
            |_caller: wasmtime::StoreContextMut<'_, SsrHostState>,
             (_self, _index): (u64, u32)|
             -> Result<(Option<u64>,), wasmtime::Error> { Ok((None,)) },
        )?;
    }

    // history: back/forward/push-state (no-ops)
    {
        let mut hist = linker.instance("tairitsu-browser:full/history@0.2.0")?;
        hist.func_wrap(
            "back",
            |_caller: wasmtime::StoreContextMut<'_, SsrHostState>,
             (_self,): (u64,)|
             -> Result<(), wasmtime::Error> { Ok(()) },
        )?;
        hist.func_wrap(
            "forward",
            |_caller: wasmtime::StoreContextMut<'_, SsrHostState>,
             (_self,): (u64,)|
             -> Result<(), wasmtime::Error> { Ok(()) },
        )?;
        hist.func_wrap(
            "push-state",
            |_caller: wasmtime::StoreContextMut<'_, SsrHostState>,
             (_self, _data, _title, _url): (u64, String, String, Option<String>)|
             -> Result<(), wasmtime::Error> { Ok(()) },
        )?;
    }

    // location: href, pathname, etc.
    {
        let mut loc = linker.instance("tairitsu-browser:full/location@0.2.0")?;
        loc.func_wrap(
            "get-href",
            |_caller: wasmtime::StoreContextMut<'_, SsrHostState>,
             (_self,): (u64,)|
             -> Result<(String,), wasmtime::Error> { Ok(("/".to_string(),)) },
        )?;
        loc.func_wrap(
            "get-pathname",
            |_caller: wasmtime::StoreContextMut<'_, SsrHostState>,
             (_self,): (u64,)|
             -> Result<(String,), wasmtime::Error> { Ok(("/".to_string(),)) },
        )?;
    }

    Ok(())
}
