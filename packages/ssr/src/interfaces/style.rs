//! Style interface implementation for SSR
//!
//! Manages CSS properties on SSR DOM nodes.

use crate::host_state::SsrHostState;

/// Add style interface to the linker
pub fn add_to_linker<T>(
    linker: &mut wasmtime::Linker<T>,
    mut get_state: impl FnMut(&mut T) -> &mut SsrHostState + Send + Sync + Copy + 'static,
) -> anyhow::Result<()>
where
    T: Send,
{
    linker.func_wrap(
        "tairitsu-browser:full/style@0.2.0",
        "set-style-property",
        move |mut caller: wasmtime::Caller<'_, T>,
              element: u64,
              property: String,
              value: String|
              -> Result<(Result<(), String>,), wasmtime::Error> {
            let state = get_state(caller.data_mut());
            let dom = &mut state.dom;

            let node = dom.get_node_mut(element);
            let Some(node) = node else {
                return Ok((Err(format!("Element not found: {}", element)),));
            };

            node.set_style_property(&property, &value);
            Ok((Ok(()),))
        },
    )?;

    linker.func_wrap(
        "tairitsu-browser:full/style@0.2.0",
        "get-style-property",
        move |mut caller: wasmtime::Caller<'_, T>,
              element: u64,
              property: String|
              -> Result<Option<String>, wasmtime::Error> {
            let state = get_state(caller.data_mut());
            let dom = &state.dom;

            let node = dom.get_node(element);
            let Some(node) = node else {
                return Ok(None);
            };

            Ok(node.get_style_property(&property))
        },
    )?;

    linker.func_wrap(
        "tairitsu-browser:full/style@0.2.0",
        "remove-style-property",
        move |mut caller: wasmtime::Caller<'_, T>,
              element: u64,
              property: String|
              -> Result<(Result<(), String>,), wasmtime::Error> {
            let state = get_state(caller.data_mut());
            let dom = &mut state.dom;

            let node = dom.get_node_mut(element);
            let Some(node) = node else {
                return Ok((Err(format!("Element not found: {}", element)),));
            };

            node.remove_style_property(&property);
            Ok((Ok(()),))
        },
    )?;

    Ok(())
}
