//! WIT linker registration for SSR
//!
//! This module registers all WIT interface implementations with the wasmtime Linker.

use crate::host_state::SsrHostState;
use crate::stubs;
use anyhow::Result;
use wasmtime::component::Linker;

/// Register all SSR WIT implementations with the linker (direct version)
///
/// This version works directly with SsrHostState.
pub fn register_ssr_imports_direct(linker: &mut Linker<SsrHostState>) -> Result<()> {
    register_core_imports(linker)?;
    stubs::register_all_stubs(linker)?;
    Ok(())
}

/// Register core DOM imports that SSR actually needs
fn register_core_imports(linker: &mut Linker<SsrHostState>) -> Result<()> {
    // Console interface
    let mut console = linker.instance("tairitsu-browser:full/console@0.2.0")?;
    console.func_wrap(
        "log",
        |_caller: wasmtime::StoreContextMut<'_, SsrHostState>, (msg,): (String,)| {
            tracing::info!("{}", msg);
            Ok(())
        },
    )?;

    console.func_wrap(
        "warn",
        |_caller: wasmtime::StoreContextMut<'_, SsrHostState>, (msg,): (String,)| {
            tracing::warn!("{}", msg);
            Ok(())
        },
    )?;

    console.func_wrap(
        "error",
        |_caller: wasmtime::StoreContextMut<'_, SsrHostState>, (msg,): (String,)| {
            tracing::error!("{}", msg);
            Ok(())
        },
    )?;

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
                .map_err(|e| wasmtime::Error::from(anyhow::anyhow!(e)))?;
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
                .map_err(|e| wasmtime::Error::from(anyhow::anyhow!(e)))?;
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

    // Style interface
    let mut style = linker.instance("tairitsu-browser:full/style@0.2.0")?;
    style.func_wrap(
        "set-style-property",
        |mut caller: wasmtime::StoreContextMut<'_, SsrHostState>,
         (element, property, value): (u64, String, String)|
         -> Result<(Result<(), String>,), wasmtime::Error> {
            let state = caller.data_mut();
            if let Some(node) = state.dom.get_node_mut(element) {
                node.set_style_property(&property, &value);
                return Ok((Ok(()),));
            }
            Ok((Err("Element not found".to_string()),))
        },
    )?;

    style.func_wrap(
        "get-style-property",
        |caller: wasmtime::StoreContextMut<'_, SsrHostState>,
         (element, property): (u64, String)|
         -> Result<(Option<String>,), wasmtime::Error> {
            let dom = &caller.data().dom;
            let value = dom
                .get_node(element)
                .and_then(|n| n.get_style_property(&property).map(|s| s.to_string()));
            Ok((value,))
        },
    )?;

    style.func_wrap(
        "remove-style-property",
        |mut caller: wasmtime::StoreContextMut<'_, SsrHostState>,
         (element, property): (u64, String)|
         -> Result<(Result<(), String>,), wasmtime::Error> {
            let state = caller.data_mut();
            if let Some(node) = state.dom.get_node_mut(element) {
                node.remove_style_property(&property);
                return Ok((Ok(()),));
            }
            Ok((Err("Element not found".to_string()),))
        },
    )?;

    // Platform helpers
    let mut platform_helpers = linker.instance("tairitsu-browser:full/platform-helpers@0.2.0")?;
    platform_helpers.func_wrap(
        "inner-width",
        |_caller: wasmtime::StoreContextMut<'_, SsrHostState>,
         (): ()|
         -> Result<(i32,), wasmtime::Error> { Ok((1920,)) },
    )?;

    platform_helpers.func_wrap(
        "inner-height",
        |_caller: wasmtime::StoreContextMut<'_, SsrHostState>,
         (): ()|
         -> Result<(i32,), wasmtime::Error> { Ok((1080,)) },
    )?;

    platform_helpers.func_wrap(
        "set-timeout",
        |_caller: wasmtime::StoreContextMut<'_, SsrHostState>,
         (_callback_id, _ms): (u64, i32)|
         -> Result<(i32,), wasmtime::Error> { Ok((1,)) },
    )?;

    platform_helpers.func_wrap(
        "clear-timeout",
        |_caller: wasmtime::StoreContextMut<'_, SsrHostState>,
         (_id,): (i32,)|
         -> Result<(), wasmtime::Error> { Ok(()) },
    )?;

    platform_helpers.func_wrap(
        "request-animation-frame",
        |_caller: wasmtime::StoreContextMut<'_, SsrHostState>,
         (_callback_id,): (u64,)|
         -> Result<(u32,), wasmtime::Error> { Ok((1,)) },
    )?;

    platform_helpers.func_wrap(
        "cancel-animation-frame",
        |_caller: wasmtime::StoreContextMut<'_, SsrHostState>,
         (_id,): (u32,)|
         -> Result<(), wasmtime::Error> { Ok(()) },
    )?;

    platform_helpers.func_wrap(
        "get-bounding-client-rect",
        |_caller: wasmtime::StoreContextMut<'_, SsrHostState>,
         (_element,): (u64,)|
         -> Result<(f64, f64, f64, f64), wasmtime::Error> { Ok((0.0, 0.0, 0.0, 0.0)) },
    )?;

    platform_helpers.func_wrap(
        "create-resize-observer",
        |_caller: wasmtime::StoreContextMut<'_, SsrHostState>,
         (_callback_id,): (u64,)|
         -> Result<(u64,), wasmtime::Error> { Ok((0,)) },
    )?;

    platform_helpers.func_wrap(
        "observe-resize",
        |_caller: wasmtime::StoreContextMut<'_, SsrHostState>,
         (_observer, _element): (u64, u64)|
         -> Result<(), wasmtime::Error> { Ok(()) },
    )?;

    platform_helpers.func_wrap(
        "unobserve-resize",
        |_caller: wasmtime::StoreContextMut<'_, SsrHostState>,
         (_observer, _element): (u64, u64)|
         -> Result<(), wasmtime::Error> { Ok(()) },
    )?;

    platform_helpers.func_wrap(
        "disconnect-resize",
        |_caller: wasmtime::StoreContextMut<'_, SsrHostState>,
         (_observer,): (u64,)|
         -> Result<(), wasmtime::Error> { Ok(()) },
    )?;

    platform_helpers.func_wrap(
        "create-mutation-observer",
        |_caller: wasmtime::StoreContextMut<'_, SsrHostState>,
         (_callback_id,): (u64,)|
         -> Result<(u64,), wasmtime::Error> { Ok((0,)) },
    )?;

    platform_helpers.func_wrap(
        "observe-mutations",
        |_caller: wasmtime::StoreContextMut<'_, SsrHostState>,
         (_observer, _element, _options): (u64, u64, Option<u64>)|
         -> Result<(), wasmtime::Error> { Ok(()) },
    )?;

    platform_helpers.func_wrap(
        "disconnect-mutation",
        |_caller: wasmtime::StoreContextMut<'_, SsrHostState>,
         (_observer,): (u64,)|
         -> Result<(), wasmtime::Error> { Ok(()) },
    )?;

    // Event target
    let mut event_target = linker.instance("tairitsu-browser:full/event-target@0.2.0")?;
    event_target.func_wrap(
        "add-event-listener",
        |_caller: wasmtime::StoreContextMut<'_, SsrHostState>,
         (_target, _event_type, _use_capture): (u64, String, bool)|
         -> Result<(Result<u64, String>,), wasmtime::Error> { Ok((Ok(1u64),)) },
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
         (_event,): (u64,)|
         -> Result<(), wasmtime::Error> { Ok(()) },
    )?;

    event_target.func_wrap(
        "stop-propagation",
        |_caller: wasmtime::StoreContextMut<'_, SsrHostState>,
         (_event,): (u64,)|
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

    // Resize observer entry
    let mut resize_observer_entry = linker.instance("tairitsu-browser:full/resize-observer-entry@0.2.0")?;
    resize_observer_entry.func_wrap(
        "get-target",
        |_caller: wasmtime::StoreContextMut<'_, SsrHostState>,
         (_self,): (u64,)|
         -> Result<(u64,), wasmtime::Error> { Ok((0,)) },
    )?;

    resize_observer_entry.func_wrap(
        "get-content-rect",
        |_caller: wasmtime::StoreContextMut<'_, SsrHostState>,
         (_self,): (u64,)|
         -> Result<(f64, f64, f64, f64), wasmtime::Error> { Ok((0.0, 0.0, 0.0, 0.0)) },
    )?;

    resize_observer_entry.func_wrap(
        "get-border-box-size",
        |_caller: wasmtime::StoreContextMut<'_, SsrHostState>,
         (_self,): (u64,)|
         -> Result<(Vec<u64>,), wasmtime::Error> { Ok((vec![],)) },
    )?;

    resize_observer_entry.func_wrap(
        "get-content-box-size",
        |_caller: wasmtime::StoreContextMut<'_, SsrHostState>,
         (_self,): (u64,)|
         -> Result<(Vec<u64>,), wasmtime::Error> { Ok((vec![],)) },
    )?;

    resize_observer_entry.func_wrap(
        "get-device-pixel-content-box-size",
        |_caller: wasmtime::StoreContextMut<'_, SsrHostState>,
         (_self,): (u64,)|
         -> Result<(Vec<u64>,), wasmtime::Error> { Ok((vec![],)) },
    )?;

    // Resize observer size
    let mut resize_observer_size = linker.instance("tairitsu-browser:full/resize-observer-size@0.2.0")?;
    resize_observer_size.func_wrap(
        "get-inline-size",
        |_caller: wasmtime::StoreContextMut<'_, SsrHostState>,
         (_self,): (u64,)|
         -> Result<(f64,), wasmtime::Error> { Ok((0.0,)) },
    )?;

    resize_observer_size.func_wrap(
        "get-block-size",
        |_caller: wasmtime::StoreContextMut<'_, SsrHostState>,
         (_self,): (u64,)|
         -> Result<(f64,), wasmtime::Error> { Ok((0.0,)) },
    )?;

    Ok(())
}
