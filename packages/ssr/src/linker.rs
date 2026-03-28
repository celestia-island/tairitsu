//! WIT linker registration for SSR
//!
//! This module registers all WIT interface implementations with the wasmtime Linker.

use crate::host_state::SsrHostState;
use crate::stubs;
use anyhow::Result;
use wasmtime::component::{HasSelf, Linker};

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
                .map_err(|e| anyhow::anyhow!(e))?;
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
                .map_err(|e| anyhow::anyhow!(e))?;
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
        |_caller: wasmtime::StoreContextMut<'_, SsrHostState>,
         (): ()|
         -> Result<(i32,), wasmtime::Error> { Ok((1920,)) },
    )?;

    window.func_wrap(
        "get-inner-height",
        |_caller: wasmtime::StoreContextMut<'_, SsrHostState>,
         (): ()|
         -> Result<(i32,), wasmtime::Error> { Ok((1080,)) },
    )?;

    Ok(())
}
