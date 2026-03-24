//! Node interface implementation for SSR
//!
//! Provides basic node operations like append-child, remove-child.

use crate::host_state::SsrHostState;

/// Add node interface to the linker
pub fn add_to_linker<T>(
    linker: &mut wasmtime::Linker<T>,
    mut get_state: impl FnMut(&mut T) -> &mut SsrHostState + Send + Sync + Copy + 'static,
) -> anyhow::Result<()>
where
    T: Send,
{
    // append-child
    linker.func_wrap(
        "tairitsu-browser:full/node@0.2.0",
        "append-child",
        move |mut caller: wasmtime::Caller<'_, T>,
              parent: u64,
              child: u64|
              -> Result<(Result<u64, String>,), wasmtime::Error> {
            let state = get_state(caller.data_mut());
            match state.dom.append_child(parent, child) {
                Ok(()) => Ok((Ok(child),)),
                Err(e) => Ok((Err(e),)),
            }
        },
    )?;

    // remove-child
    linker.func_wrap(
        "tairitsu-browser:full/node@0.2.0",
        "remove-child",
        move |mut caller: wasmtime::Caller<'_, T>,
              parent: u64,
              child: u64|
              -> Result<(Result<u64, String>,), wasmtime::Error> {
            let state = get_state(caller.data_mut());
            match state.dom.remove_child(parent, child) {
                Ok(()) => Ok((Ok(child),)),
                Err(e) => Ok((Err(e),)),
            }
        },
    )?;

    // get-node-type
    linker.func_wrap(
        "tairitsu-browser:full/node@0.2.0",
        "get-node-type",
        move |mut caller: wasmtime::Caller<'_, T>, handle: u64| -> Result<u16, wasmtime::Error> {
            let state = get_state(caller.data_mut());
            if let Some(node) = state.dom.get_node(handle) {
                let node_type = match &node.kind {
                    crate::virtual_dom::SsrNodeKind::Element { .. } => 1, // ELEMENT_NODE
                    crate::virtual_dom::SsrNodeKind::Text { .. } => 3,     // TEXT_NODE
                };
                Ok(node_type)
            } else {
                Ok(0) // Invalid node
            }
        },
    )?;

    // get-node-name
    linker.func_wrap(
        "tairitsu-browser:full/node@0.2.0",
        "get-node-name",
        move |mut caller: wasmtime::Caller<'_, T>, handle: u64| -> Result<String, wasmtime::Error> {
            let state = get_state(caller.data_mut());
            if let Some(node) = state.dom.get_node(handle) {
                let name = match &node.kind {
                    crate::virtual_dom::SsrNodeKind::Element { tag, .. } => tag.clone(),
                    crate::virtual_dom::SsrNodeKind::Text { .. } => "#text".to_string(),
                };
                Ok(name)
            } else {
                Ok("".to_string())
            }
        },
    )?;

    // get-base-uri - returns empty string
    linker.func_wrap(
        "tairitsu-browser:full/node@0.2.0",
        "get-base-uri",
        move |_caller: wasmtime::Caller<'_, T>, _handle: u64| -> Result<String, wasmtime::Error> {
            Ok("about:blank".to_string())
        },
    )?;

    // get-is-connected - returns true (all nodes are connected in SSR)
    linker.func_wrap(
        "tairitsu-browser:full/node@0.2.0",
        "get-is-connected",
        move |_caller: wasmtime::Caller<'_, T>, _handle: u64| -> Result<bool, wasmtime::Error> {
            Ok(true)
        },
    )?;

    // get-owner-document - returns None
    linker.func_wrap(
        "tairitsu-browser:full/node@0.2.0",
        "get-owner-document",
        move |_caller: wasmtime::Caller<'_, T>, _handle: u64| -> Result<Option<u64>, wasmtime::Error> {
            Ok(None)
        },
    )?;

    // get-root-node - returns same node (it is its own root in SSR)
    linker.func_wrap(
        "tairitsu-browser:full/node@0.2.0",
        "get-root-node",
        move |_caller: wasmtime::Caller<'_, T>, handle: u64, _options: Option<u64>| -> Result<u64, wasmtime::Error> {
            Ok(handle)
        },
    )?;

    // get-parent-node
    linker.func_wrap(
        "tairitsu-browser:full/node@0.2.0",
        "get-parent-node",
        move |mut caller: wasmtime::Caller<'_, T>, handle: u64| -> Result<Option<u64>, wasmtime::Error> {
            let state = get_state(caller.data_mut());
            if let Some(node) = state.dom.get_node(handle) {
                Ok(node.parent)
            } else {
                Ok(None)
            }
        },
    )?;

    // get-parent-element - returns same as parent-node for SSR
    linker.func_wrap(
        "tairitsu-browser:full/node@0.2.0",
        "get-parent-element",
        move |mut caller: wasmtime::Caller<'_, T>, handle: u64| -> Result<Option<u64>, wasmtime::Error> {
            let state = get_state(caller.data_mut());
            if let Some(node) = state.dom.get_node(handle) {
                // Only return parent if it's an element (not text node's parent)
                if let Some(parent) = node.parent {
                    if let Some(parent_node) = state.dom.get_node(parent) {
                        if matches!(parent_node.kind, crate::virtual_dom::SsrNodeKind::Element { .. }) {
                            return Ok(Some(parent));
                        }
                    }
                }
            }
            Ok(None)
        },
    )?;

    // has-child-nodes
    linker.func_wrap(
        "tairitsu-browser:full/node@0.2.0",
        "has-child-nodes",
        move |mut caller: wasmtime::Caller<'_, T>, handle: u64| -> Result<bool, wasmtime::Error> {
            let state = get_state(caller.data_mut());
            if let Some(node) = state.dom.get_node(handle) {
                Ok(!node.children.is_empty())
            } else {
                Ok(false)
            }
        },
    )?;

    // get-child-nodes - returns empty list (stub)
    linker.func_wrap(
        "tairitsu-browser:full/node@0.2.0",
        "get-child-nodes",
        move |mut caller: wasmtime::Caller<'_, T>, handle: u64| -> Result<u64, wasmtime::Error> {
            let state = get_state(caller.data_mut());
            // Return a dummy handle for the NodeList
            // In a full implementation, we'd create a proper NodeList
            Ok(state.dom.create_element("node-list", None))
        },
    )?;

    // get-first-child
    linker.func_wrap(
        "tairitsu-browser:full/node@0.2.0",
        "get-first-child",
        move |mut caller: wasmtime::Caller<'_, T>, handle: u64| -> Result<Option<u64>, wasmtime::Error> {
            let state = get_state(caller.data_mut());
            if let Some(node) = state.dom.get_node(handle) {
                Ok(node.children.first().copied())
            } else {
                Ok(None)
            }
        },
    )?;

    // get-last-child
    linker.func_wrap(
        "tairitsu-browser:full/node@0.2.0",
        "get-last-child",
        move |mut caller: wasmtime::Caller<'_, T>, handle: u64| -> Result<Option<u64>, wasmtime::Error> {
            let state = get_state(caller.data_mut());
            if let Some(node) = state.dom.get_node(handle) {
                Ok(node.children.last().copied())
            } else {
                Ok(None)
            }
        },
    )?;

    // get-next-sibling - stub (returns None)
    linker.func_wrap(
        "tairitsu-browser:full/node@0.2.0",
        "get-next-sibling",
        move |_caller: wasmtime::Caller<'_, T>, _handle: u64| -> Result<Option<u64>, wasmtime::Error> {
            Ok(None)
        },
    )?;

    // get-previous-sibling - stub (returns None)
    linker.func_wrap(
        "tairitsu-browser:full/node@0.2.0",
        "get-previous-sibling",
        move |_caller: wasmtime::Caller<'_, T>, _handle: u64| -> Result<Option<u64>, wasmtime::Error> {
            Ok(None)
        },
    )?;

    // clone-node - returns same handle (shallow clone stub)
    linker.func_wrap(
        "tairitsu-browser:full/node@0.2.0",
        "clone-node",
        move |mut caller: wasmtime::Caller<'_, T>, handle: u64, _deep: bool| -> Result<u64, wasmtime::Error> {
            let state = get_state(caller.data_mut());
            // Create a clone by copying the node
            if let Some(node) = state.dom.get_node(handle) {
                let new_handle = state.dom.create_element("div", None); // Placeholder
                // Copy attributes
                if let Some(new_node) = state.dom.get_node_mut(new_handle) {
                    for (name, value) in &node.attributes {
                        new_node.set_attribute(name, value);
                    }
                }
                Ok(new_handle)
            } else {
                Ok(handle)
            }
        },
    )?;

    // normalize - no-op
    linker.func_wrap(
        "tairitsu-browser:full/node@0.2.0",
        "normalize",
        move |_caller: wasmtime::Caller<'_, T>, _handle: u64| {
            Ok(())
        },
    )?;

    // insert-before - same as append-child for SSR
    linker.func_wrap(
        "tairitsu-browser:full/node@0.2.0",
        "insert-before",
        move |mut caller: wasmtime::Caller<'_, T>,
              parent: u64,
              child: u64,
              _reference_child: Option<u64>|
              -> Result<(Result<u64, String>,), wasmtime::Error> {
            let state = get_state(caller.data_mut());
            match state.dom.append_child(parent, child) {
                Ok(()) => Ok((Ok(child),)),
                Err(e) => Ok((Err(e),)),
            }
        },
    )?;

    // replace-child
    linker.func_wrap(
        "tairitsu-browser:full/node@0.2.0",
        "replace-child",
        move |mut caller: wasmtime::Caller<'_, T>,
              parent: u64,
              new_child: u64,
              _old_child: u64|
              -> Result<(Result<u64, String>,), wasmtime::Error> {
            let state = get_state(caller.data_mut());
            // Simply append for now
            match state.dom.append_child(parent, new_child) {
                Ok(()) => Ok((Ok(new_child),)),
                Err(e) => Ok((Err(e),)),
            }
        },
    )?;

    // contains
    linker.func_wrap(
        "tairitsu-browser:full/node@0.2.0",
        "contains",
        move |mut caller: wasmtime::Caller<'_, T>, parent: u64, child: u64| -> Result<bool, wasmtime::Error> {
            let state = get_state(caller.data_mut());
            // Simple check - is child a descendant of parent
            let mut current = state.dom.get_node(child).and_then(|n| n.parent);
            while let Some(handle) = current {
                if handle == parent {
                    return Ok(true);
                }
                current = state.dom.get_node(handle).and_then(|n| n.parent);
            }
            Ok(false)
        },
    )?;

    // lookup-prefix - returns None
    linker.func_wrap(
        "tairitsu-browser:full/node@0.2.0",
        "lookup-prefix",
        move |_caller: wasmtime::Caller<'_, T>, _handle: u64, _namespace: Option<String>| -> Result<Option<String>, wasmtime::Error> {
            Ok(None)
        },
    )?;

    // lookup-namespace-uri - returns None
    linker.func_wrap(
        "tairitsu-browser:full/node@0.2.0",
        "lookup-namespace-uri",
        move |_caller: wasmtime::Caller<'_, T>, _handle: u64, _prefix: Option<String>| -> Result<Option<String>, wasmtime::Error> {
            Ok(None)
        },
    )?;

    // is-default-namespace - returns true
    linker.func_wrap(
        "tairitsu-browser:full/node@0.2.0",
        "is-default-namespace",
        move |_caller: wasmtime::Caller<'_, T>, _handle: u64, _namespace: Option<String>| -> Result<bool, wasmtime::Error> {
            Ok(true)
        },
    )?;

    // compare-document-position - returns 0 (same position)
    linker.func_wrap(
        "tairitsu-browser:full/node@0.2.0",
        "compare-document-position",
        move |_caller: wasmtime::Caller<'_, T>, _handle: u64, _other: u64| -> Result<u16, wasmtime::Error> {
            Ok(0)
        },
    )?;

    // is-equal-node - simple check
    linker.func_wrap(
        "tairitsu-browser:full/node@0.2.0",
        "is-equal-node",
        move |mut caller: wasmtime::Caller<'_, T>, handle: u64, other: u64| -> Result<bool, wasmtime::Error> {
            let state = get_state(caller.data_mut());
            if let (Some(a), Some(b)) = (state.dom.get_node(handle), state.dom.get_node(other)) {
                // Simple check - same tag name
                Ok(a.tag_name() == b.tag_name())
            } else {
                Ok(false)
            }
        },
    )?;

    // is-same-node - simple handle comparison
    linker.func_wrap(
        "tairitsu-browser:full/node@0.2.0",
        "is-same-node",
        move |_caller: wasmtime::Caller<'_, T>, handle: u64, other: u64| -> Result<bool, wasmtime::Error> {
            Ok(handle == other)
        },
    )?;

    Ok(())
}
