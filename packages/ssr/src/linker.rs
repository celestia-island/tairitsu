//! WIT linker registration for SSR
//!
//! This module registers all WIT interface implementations with the wasmtime Linker.

use crate::host_state::SsrHostState;
use anyhow::Result;
use wasmtime::component::Linker;

/// Register all SSR WIT implementations with the linker (direct version)
///
/// This version works directly with SsrHostState.
pub fn register_ssr_imports_direct(linker: &mut Linker<SsrHostState>) -> Result<()> {
    register_core_imports(linker)?;
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
        |mut caller: wasmtime::StoreContextMut<'_, SsrHostState>, (local_name, _options): (String, Option<String>)|
         -> Result<(u64,), wasmtime::Error> {
            let state = caller.data_mut();
            Ok((state.dom.create_element(&local_name, None),))
        },
    )?;

    document.func_wrap(
        "create-text-node",
        |mut caller: wasmtime::StoreContextMut<'_, SsrHostState>, (data,): (String,)|
         -> Result<(u64,), wasmtime::Error> {
            let state = caller.data_mut();
            Ok((state.dom.create_text_node(&data),))
        },
    )?;

    document.func_wrap(
        "get-body",
        |mut caller: wasmtime::StoreContextMut<'_, SsrHostState>, (): ()| -> Result<(Option<u64>,), wasmtime::Error> {
            let state = caller.data_mut();
            Ok((Some(state.dom.body_handle()),))
        },
    )?;

    document.func_wrap(
        "get-head",
        |mut caller: wasmtime::StoreContextMut<'_, SsrHostState>, (): ()| -> Result<(Option<u64>,), wasmtime::Error> {
            let state = caller.data_mut();
            Ok((Some(state.dom.head_handle()),))
        },
    )?;

    // Node interface
    let mut node = linker.instance("tairitsu-browser:full/node@0.2.0")?;
    node.func_wrap(
        "append-child",
        |mut caller: wasmtime::StoreContextMut<'_, SsrHostState>, (parent, child): (u64, u64)|
         -> Result<(Result<u64, String>,), wasmtime::Error> {
            let state = caller.data_mut();
            match state.dom.append_child(parent, child) {
                Ok(()) => Ok((Ok(child),)),
                Err(e) => Ok((Err(e),)),
            }
        },
    )?;

    // Element interface
    let mut element = linker.instance("tairitsu-browser:full/element@0.2.0")?;
    element.func_wrap(
        "set-attribute",
        |mut caller: wasmtime::StoreContextMut<'_, SsrHostState>, (self_handle, name, value): (u64, String, String)|
         -> Result<(Result<(), String>,), wasmtime::Error> {
            let state = caller.data_mut();
            if let Some(node) = state.dom.get_node_mut(self_handle) {
                node.set_attribute(&name, &value);
                return Ok((Ok(()),));
            }
            Ok((Err("Element not found".to_string()),))
        },
    )?;

    element.func_wrap(
        "set-class-name",
        |mut caller: wasmtime::StoreContextMut<'_, SsrHostState>, (self_handle, value): (u64, String)| {
            let state = caller.data_mut();
            if let Some(node) = state.dom.get_node_mut(self_handle) {
                node.set_class(&value);
            }
            Ok(())
        },
    )?;

    // Style interface
    let mut style = linker.instance("tairitsu-browser:full/style@0.2.0")?;
    style.func_wrap(
        "set-style-property",
        |mut caller: wasmtime::StoreContextMut<'_, SsrHostState>, (element, property, value): (u64, String, String)|
         -> Result<(Result<(), String>,), wasmtime::Error> {
            let state = caller.data_mut();
            if let Some(node) = state.dom.get_node_mut(element) {
                node.set_style_property(&property, &value);
                return Ok((Ok(()),));
            }
            Ok((Err("Element not found".to_string()),))
        },
    )?;

    // Platform helpers
    let mut platform_helpers = linker.instance("tairitsu-browser:full/platform-helpers@0.2.0")?;
    platform_helpers.func_wrap(
        "inner-width",
        |_caller: wasmtime::StoreContextMut<'_, SsrHostState>, (): ()| -> Result<(i32,), wasmtime::Error> {
            Ok((1920,))
        },
    )?;

    platform_helpers.func_wrap(
        "inner-height",
        |_caller: wasmtime::StoreContextMut<'_, SsrHostState>, (): ()| -> Result<(i32,), wasmtime::Error> {
            Ok((1080,))
        },
    )?;

    platform_helpers.func_wrap(
        "set-timeout",
        |_caller: wasmtime::StoreContextMut<'_, SsrHostState>, (_callback_id, _ms): (u64, i32)|
         -> Result<(i32,), wasmtime::Error> { Ok((1,)) },
    )?;

    platform_helpers.func_wrap(
        "clear-timeout",
        |_caller: wasmtime::StoreContextMut<'_, SsrHostState>, (_id,): (i32,)| -> Result<(), wasmtime::Error> {
            Ok(())
        },
    )?;

    platform_helpers.func_wrap(
        "request-animation-frame",
        |_caller: wasmtime::StoreContextMut<'_, SsrHostState>, (_callback_id,): (u64,)|
         -> Result<(u32,), wasmtime::Error> { Ok((1,)) },
    )?;

    platform_helpers.func_wrap(
        "cancel-animation-frame",
        |_caller: wasmtime::StoreContextMut<'_, SsrHostState>, (_id,): (u32,)| -> Result<(), wasmtime::Error> {
            Ok(())
        },
    )?;

    platform_helpers.func_wrap(
        "get-bounding-client-rect",
        |_caller: wasmtime::StoreContextMut<'_, SsrHostState>, (_element,): (u64,)|
         -> Result<(f64, f64, f64, f64), wasmtime::Error> {
            Ok((0.0, 0.0, 0.0, 0.0))
        },
    )?;

    // Event target
    let mut event_target = linker.instance("tairitsu-browser:full/event-target@0.2.0")?;
    event_target.func_wrap(
        "add-event-listener",
        |_caller: wasmtime::StoreContextMut<'_, SsrHostState>, (_target, _event_type, _use_capture): (u64, String, bool)|
         -> Result<(Result<u64, String>,), wasmtime::Error> { Ok((Ok(1u64),)) },
    )?;

    event_target.func_wrap(
        "remove-event-listener",
        |_caller: wasmtime::StoreContextMut<'_, SsrHostState>, (_target, _listener_id): (u64, u64)|
         -> Result<(Result<(), String>,), wasmtime::Error> { Ok((Ok(()),)) },
    )?;

    event_target.func_wrap(
        "prevent-default",
        |_caller: wasmtime::StoreContextMut<'_, SsrHostState>, (_event,): (u64,)| -> Result<(), wasmtime::Error> {
            Ok(())
        },
    )?;

    event_target.func_wrap(
        "stop-propagation",
        |_caller: wasmtime::StoreContextMut<'_, SsrHostState>, (_event,): (u64,)| -> Result<(), wasmtime::Error> {
            Ok(())
        },
    )?;

    // Window
    let mut window = linker.instance("tairitsu-browser:full/window@0.2.0")?;
    window.func_wrap(
        "get-inner-width",
        |_caller: wasmtime::StoreContextMut<'_, SsrHostState>, (): ()| -> Result<(i32,), wasmtime::Error> {
            Ok((1920,))
        },
    )?;

    window.func_wrap(
        "get-inner-height",
        |_caller: wasmtime::StoreContextMut<'_, SsrHostState>, (): ()| -> Result<(i32,), wasmtime::Error> {
            Ok((1080,))
        },
    )?;

    Ok(())
}
