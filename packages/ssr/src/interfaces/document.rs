//! Document interface implementation for SSR
//!
//! Provides DOM creation and query functions.

use crate::host_state::SsrHostState;

/// Add document interface to the linker
pub fn add_to_linker<T>(
    linker: &mut wasmtime::Linker<T>,
    mut get_state: impl FnMut(&mut T) -> &mut SsrHostState + Send + Sync + Copy + 'static,
) -> anyhow::Result<()>
where
    T: Send,
{
    // create-element
    linker.func_wrap(
        "tairitsu-browser:full/document@0.2.0",
        "create-element",
        move |mut caller: wasmtime::Caller<'_, T>,
              local_name: String,
              _options: Option<String>|
              -> Result<u64, wasmtime::Error> {
            let state = get_state(caller.data_mut());
            Ok(state.dom.create_element(&local_name, None))
        },
    )?;

    // create-element-ns
    linker.func_wrap(
        "tairitsu-browser:full/document@0.2.0",
        "create-element-ns",
        move |mut caller: wasmtime::Caller<'_, T>,
              namespace: Option<String>,
              qualified_name: String,
              _options: Option<String>|
              -> Result<u64, wasmtime::Error> {
            let state = get_state(caller.data_mut());
            Ok(state.dom.create_element(
                &qualified_name,
                namespace.as_ref().map(|s| s.as_str()),
            ))
        },
    )?;

    // create-text-node
    linker.func_wrap(
        "tairitsu-browser:full/document@0.2.0",
        "create-text-node",
        move |mut caller: wasmtime::Caller<'_, T>, data: String| -> Result<u64, wasmtime::Error> {
            let state = get_state(caller.data_mut());
            Ok(state.dom.create_text_node(&data))
        },
    )?;

    // get-body
    linker.func_wrap(
        "tairitsu-browser:full/document@0.2.0",
        "get-body",
        move |mut caller: wasmtime::Caller<'_, T>| -> Result<Option<u64>, wasmtime::Error> {
            let state = get_state(caller.data_mut());
            Ok(Some(state.dom.body_handle()))
        },
    )?;

    // get-head
    linker.func_wrap(
        "tairitsu-browser:full/document@0.2.0",
        "get-head",
        move |mut caller: wasmtime::Caller<'_, T>| -> Result<Option<u64>, wasmtime::Error> {
            let state = get_state(caller.data_mut());
            Ok(Some(state.dom.head_handle()))
        },
    )?;

    // get-element-by-id (stub in non-element-parent-node, but we need it for document)
    // This is a simple implementation - the full interface has many more functions
    // For the rest, we'll provide stub implementations that return sensible defaults

    // For now, let's stub the remaining functions that the WIT expects
    // Most of these will be implemented as stubs that return empty/default values

    // element-from-point - returns None in SSR
    linker.func_wrap(
        "tairitsu-browser:full/document@0.2.0",
        "element-from-point",
        move |_caller: wasmtime::Caller<'_, T>, _x: f64, _y: f64| -> Result<Option<u64>, wasmtime::Error> {
            Ok(None)
        },
    )?;

    // elements-from-point - returns empty list
    linker.func_wrap(
        "tairitsu-browser:full/document@0.2.0",
        "elements-from-point",
        move |_caller: wasmtime::Caller<'_, T>, _x: f64, _y: f64| -> Result<Vec<u64>, wasmtime::Error> {
            Ok(Vec::new())
        },
    )?;

    // caret-position-from-point - returns None
    linker.func_wrap(
        "tairitsu-browser:full/document@0.2.0",
        "caret-position-from-point",
        move |_caller: wasmtime::Caller<'_, T>, _x: f64, _y: f64, _options: Option<u64>| -> Result<Option<u64>, wasmtime::Error> {
            Ok(None)
        },
    )?;

    // get-scrolling-element - returns None
    linker.func_wrap(
        "tairitsu-browser:full/document@0.2.0",
        "get-scrolling-element",
        move |_caller: wasmtime::Caller<'_, T>| -> Result<Option<u64>, wasmtime::Error> {
            Ok(None)
        },
    )?;

    // get-implementation - returns dummy handle
    linker.func_wrap(
        "tairitsu-browser:full/document@0.2.0",
        "get-implementation",
        move |_caller: wasmtime::Caller<'_, T>| -> Result<u64, wasmtime::Error> {
            Ok(1) // Dummy handle
        },
    )?;

    // get-url - returns empty string
    linker.func_wrap(
        "tairitsu-browser:full/document@0.2.0",
        "get-url",
        move |_caller: wasmtime::Caller<'_, T>| -> Result<String, wasmtime::Error> {
            Ok("about:blank".to_string())
        },
    )?;

    // get-document-uri - returns empty string
    linker.func_wrap(
        "tairitsu-browser:full/document@0.2.0",
        "get-document-uri",
        move |_caller: wasmtime::Caller<'_, T>| -> Result<String, wasmtime::Error> {
            Ok("about:blank".to_string())
        },
    )?;

    // get-compat-mode - returns "CSS1Compat"
    linker.func_wrap(
        "tairitsu-browser:full/document@0.2.0",
        "get-compat-mode",
        move |_caller: wasmtime::Caller<'_, T>| -> Result<String, wasmtime::Error> {
            Ok("CSS1Compat".to_string())
        },
    )?;

    // get-character-set - returns "UTF-8"
    linker.func_wrap(
        "tairitsu-browser:full/document@0.2.0",
        "get-character-set",
        move |_caller: wasmtime::Caller<'_, T>| -> Result<String, wasmtime::Error> {
            Ok("UTF-8".to_string())
        },
    )?;

    // get-content-type - returns "text/html"
    linker.func_wrap(
        "tairitsu-browser:full/document@0.2.0",
        "get-content-type",
        move |_caller: wasmtime::Caller<'_, T>| -> Result<String, wasmtime::Error> {
            Ok("text/html".to_string())
        },
    )?;

    // get-doctype - returns None
    linker.func_wrap(
        "tairitsu-browser:full/document@0.2.0",
        "get-doctype",
        move |_caller: wasmtime::Caller<'_, T>| -> Result<Option<u64>, wasmtime::Error> {
            Ok(None)
        },
    )?;

    // get-document-element - returns None (no html element handle exposed)
    linker.func_wrap(
        "tairitsu-browser:full/document@0.2.0",
        "get-document-element",
        move |_caller: wasmtime::Caller<'_, T>| -> Result<Option<u64>, wasmtime::Error> {
            Ok(None)
        },
    )?;

    // create-document-fragment - returns dummy handle
    linker.func_wrap(
        "tairitsu-browser:full/document@0.2.0",
        "create-document-fragment",
        move |mut caller: wasmtime::Caller<'_, T>| -> Result<u64, wasmtime::Error> {
            let state = get_state(caller.data_mut());
            Ok(state.dom.create_element("fragment", None))
        },
    )?;

    // create-comment - returns dummy handle
    linker.func_wrap(
        "tairitsu-browser:full/document@0.2.0",
        "create-comment",
        move |mut caller: wasmtime::Caller<'_, T>| -> Result<u64, wasmtime::Error> {
            let state = get_state(caller.data_mut());
            // Create a text node as a proxy for comments
            Ok(state.dom.create_text_node(""))
        },
    )?;

    Ok(())
}
