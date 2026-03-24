//! Element interface implementation for SSR
//!
//! Provides element-specific operations like set-attribute, set-class.

use crate::host_state::SsrHostState;

/// Add element interface to the linker
pub fn add_to_linker<T>(
    linker: &mut wasmtime::Linker<T>,
    mut get_state: impl FnMut(&mut T) -> &mut SsrHostState + Send + Sync + Copy + 'static,
) -> anyhow::Result<()>
where
    T: Send,
{
    // get-client-rects - returns dummy handle
    linker.func_wrap(
        "tairitsu-browser:full/element@0.2.0",
        "get-client-rects",
        move |_caller: wasmtime::Caller<'_, T>, _self_handle: u64| -> Result<u64, wasmtime::Error> {
            Ok(1) // Dummy DOMRectList handle
        },
    )?;

    // get-bounding-client-rect - returns zero rectangle
    linker.func_wrap(
        "tairitsu-browser:full/element@0.2.0",
        "get-bounding-client-rect",
        move |_caller: wasmtime::Caller<'_, T>, _self_handle: u64| -> Result<(f64, f64, f64, f64), wasmtime::Error> {
            Ok((0.0, 0.0, 0.0, 0.0))
        },
    )?;

    // check-visibility - returns true
    linker.func_wrap(
        "tairitsu-browser:full/element@0.2.0",
        "check-visibility",
        move |_caller: wasmtime::Caller<'_, T>, _self_handle: u64, _options: Option<u64>| -> Result<bool, wasmtime::Error> {
            Ok(true)
        },
    )?;

    // scroll-into-view - no-op, returns dummy handle
    linker.func_wrap(
        "tairitsu-browser:full/element@0.2.0",
        "scroll-into-view",
        move |_caller: wasmtime::Caller<'_, T>, _self_handle: u64, _arg: Option<bool>| -> Result<u64, wasmtime::Error> {
            Ok(1) // Dummy void handle
        },
    )?;

    // scroll - no-op, returns dummy handle
    linker.func_wrap(
        "tairitsu-browser:full/element@0.2.0",
        "scroll",
        move |_caller: wasmtime::Caller<'_, T>, _self_handle: u64, _options: Option<u64>| -> Result<u64, wasmtime::Error> {
            Ok(1)
        },
    )?;

    // scroll-to - no-op, returns dummy handle
    linker.func_wrap(
        "tairitsu-browser:full/element@0.2.0",
        "scroll-to",
        move |_caller: wasmtime::Caller<'_, T>, _self_handle: u64, _options: Option<u64>| -> Result<u64, wasmtime::Error> {
            Ok(1)
        },
    )?;

    // scroll-by - no-op, returns dummy handle
    linker.func_wrap(
        "tairitsu-browser:full/element@0.2.0",
        "scroll-by",
        move |_caller: wasmtime::Caller<'_, T>, _self_handle: u64, _options: Option<u64>| -> Result<u64, wasmtime::Error> {
            Ok(1)
        },
    )?;

    // get-scroll-top - returns 0
    linker.func_wrap(
        "tairitsu-browser:full/element@0.2.0",
        "get-scroll-top",
        move |_caller: wasmtime::Caller<'_, T>, _self_handle: u64| -> Result<f64, wasmtime::Error> {
            Ok(0.0)
        },
    )?;

    // set-scroll-top - no-op
    linker.func_wrap(
        "tairitsu-browser:full/element@0.2.0",
        "set-scroll-top",
        move |_caller: wasmtime::Caller<'_, T>, _self_handle: u64, _value: f64| {
            Ok(())
        },
    )?;

    // get-scroll-left - returns 0
    linker.func_wrap(
        "tairitsu-browser:full/element@0.2.0",
        "get-scroll-left",
        move |_caller: wasmtime::Caller<'_, T>, _self_handle: u64| -> Result<f64, wasmtime::Error> {
            Ok(0.0)
        },
    )?;

    // set-scroll-left - no-op
    linker.func_wrap(
        "tairitsu-browser:full/element@0.2.0",
        "set-scroll-left",
        move |_caller: wasmtime::Caller<'_, T>, _self_handle: u64, _value: f64| {
            Ok(())
        },
    )?;

    // get-scroll-width - returns 0
    linker.func_wrap(
        "tairitsu-browser:full/element@0.2.0",
        "get-scroll-width",
        move |_caller: wasmtime::Caller<'_, T>, _self_handle: u64| -> Result<f64, wasmtime::Error> {
            Ok(0.0)
        },
    )?;

    // get-scroll-height - returns 0
    linker.func_wrap(
        "tairitsu-browser:full/element@0.2.0",
        "get-scroll-height",
        move |_caller: wasmtime::Caller<'_, T>, _self_handle: u64| -> Result<f64, wasmtime::Error> {
            Ok(0.0)
        },
    )?;

    // get-client-rects-empty - returns true
    linker.func_wrap(
        "tairitsu-browser:full/element@0.2.0",
        "get-client-rects-empty",
        move |_caller: wasmtime::Caller<'_, T>, _self_handle: u64| -> Result<bool, wasmtime::Error> {
            Ok(true)
        },
    )?;

    // get-namespace-uri - returns empty string
    linker.func_wrap(
        "tairitsu-browser:full/element@0.2.0",
        "get-namespace-uri",
        move |_caller: wasmtime::Caller<'_, T>, _self_handle: u64| -> Result<Option<String>, wasmtime::Error> {
            Ok(None)
        },
    )?;

    // get-prefix - returns None
    linker.func_wrap(
        "tairitsu-browser:full/element@0.2.0",
        "get-prefix",
        move |_caller: wasmtime::Caller<'_, T>, _self_handle: u64| -> Result<Option<String>, wasmtime::Error> {
            Ok(None)
        },
    )?;

    // get-local-name - returns tag name
    linker.func_wrap(
        "tairitsu-browser:full/element@0.2.0",
        "get-local-name",
        move |mut caller: wasmtime::Caller<'_, T>, self_handle: u64| -> Result<String, wasmtime::Error> {
            let state = get_state(caller.data_mut());
            if let Some(node) = state.dom.get_node(self_handle) {
                if let Some(tag) = node.tag_name() {
                    return Ok(tag);
                }
            }
            Ok("".to_string())
        },
    )?;

    // get-tag-name - returns tag name
    linker.func_wrap(
        "tairitsu-browser:full/element@0.2.0",
        "get-tag-name",
        move |mut caller: wasmtime::Caller<'_, T>, self_handle: u64| -> Result<String, wasmtime::Error> {
            let state = get_state(caller.data_mut());
            if let Some(node) = state.dom.get_node(self_handle) {
                if let Some(tag) = node.tag_name() {
                    return Ok(tag.to_uppercase());
                }
            }
            Ok("".to_string())
        },
    )?;

    // get-id - returns id attribute
    linker.func_wrap(
        "tairitsu-browser:full/element@0.2.0",
        "get-id",
        move |mut caller: wasmtime::Caller<'_, T>, self_handle: u64| -> Result<String, wasmtime::Error> {
            let state = get_state(caller.data_mut());
            if let Some(node) = state.dom.get_node(self_handle) {
                if let Some(id) = node.get_attribute("id") {
                    return Ok(id);
                }
            }
            Ok("".to_string())
        },
    )?;

    // set-id - sets id attribute
    linker.func_wrap(
        "tairitsu-browser:full/element@0.2.0",
        "set-id",
        move |mut caller: wasmtime::Caller<'_, T>, self_handle: u64, value: String| {
            let state = get_state(caller.data_mut());
            if let Some(node) = state.dom.get_node_mut(self_handle) {
                node.set_attribute("id", &value);
            }
            Ok(())
        },
    )?;

    // get-class-name - returns class
    linker.func_wrap(
        "tairitsu-browser:full/element@0.2.0",
        "get-class-name",
        move |mut caller: wasmtime::Caller<'_, T>, self_handle: u64| -> Result<String, wasmtime::Error> {
            let state = get_state(caller.data_mut());
            if let Some(node) = state.dom.get_node(self_handle) {
                return Ok(node.class.clone());
            }
            Ok("".to_string())
        },
    )?;

    // set-class-name - sets class
    linker.func_wrap(
        "tairitsu-browser:full/element@0.2.0",
        "set-class-name",
        move |mut caller: wasmtime::Caller<'_, T>, self_handle: u64, value: String| {
            let state = get_state(caller.data_mut());
            if let Some(node) = state.dom.get_node_mut(self_handle) {
                node.set_class(&value);
            }
            Ok(())
        },
    )?;

    // get-class-list - returns dummy handle
    linker.func_wrap(
        "tairitsu-browser:full/element@0.2.0",
        "get-class-list",
        move |_caller: wasmtime::Caller<'_, T>, _self_handle: u64| -> Result<u64, wasmtime::Error> {
            Ok(1) // Dummy DOMTokenList handle
        },
    )?;

    // get-attributes - returns dummy handle
    linker.func_wrap(
        "tairitsu-browser:full/element@0.2.0",
        "get-attributes",
        move |_caller: wasmtime::Caller<'_, T>, _self_handle: u64| -> Result<u64, wasmtime::Error> {
            Ok(1) // Dummy NamedNodeMap handle
        },
    )?;

    // get-attribute-names - returns empty list
    linker.func_wrap(
        "tairitsu-browser:full/element@0.2.0",
        "get-attribute-names",
        move |mut caller: wasmtime::Caller<'_, T>, self_handle: u64| -> Result<Vec<String>, wasmtime::Error> {
            let state = get_state(caller.data_mut());
            if let Some(node) = state.dom.get_node(self_handle) {
                let names: Vec<String> = node.attributes.iter().map(|(n, _)| n.clone()).collect();
                return Ok(names);
            }
            Ok(Vec::new())
        },
    )?;

    // get-attribute - returns attribute value
    linker.func_wrap(
        "tairitsu-browser:full/element@0.2.0",
        "get-attribute",
        move |mut caller: wasmtime::Caller<'_, T>,
              self_handle: u64,
              name: String|
              -> Result<Option<String>, wasmtime::Error> {
            let state = get_state(caller.data_mut());
            if let Some(node) = state.dom.get_node(self_handle) {
                if let Some(value) = node.get_attribute(&name) {
                    return Ok(Some(value));
                }
            }
            Ok(None)
        },
    )?;

    // get-attribute-ns - returns None
    linker.func_wrap(
        "tairitsu-browser:full/element@0.2.0",
        "get-attribute-ns",
        move |_caller: wasmtime::Caller<'_, T>, _self_handle: u64, _namespace: Option<String>, _local_name: String>
              -> Result<Option<String>, wasmtime::Error> {
            Ok(None)
        },
    )?;

    // set-attribute - sets attribute
    linker.func_wrap(
        "tairitsu-browser:full/element@0.2.0",
        "set-attribute",
        move |mut caller: wasmtime::Caller<'_, T>,
              self_handle: u64,
              name: String,
              value: String|
              -> Result<(Result<(), String>,), wasmtime::Error> {
            let state = get_state(caller.data_mut());
            if let Some(node) = state.dom.get_node_mut(self_handle) {
                node.set_attribute(&name, &value);
                return Ok((Ok(()),));
            }
            Ok((Err("Element not found".to_string()),))
        },
    )?;

    // set-attribute-ns - sets attribute (ignoring namespace)
    linker.func_wrap(
        "tairitsu-browser:full/element@0.2.0",
        "set-attribute-ns",
        move |mut caller: wasmtime::Caller<'_, T>,
              self_handle: u64,
              _namespace: Option<String>,
              name: String,
              value: String|
              -> Result<(Result<(), String>,), wasmtime::Error> {
            let state = get_state(caller.data_mut());
            if let Some(node) = state.dom.get_node_mut(self_handle) {
                node.set_attribute(&name, &value);
                return Ok((Ok(()),));
            }
            Ok((Err("Element not found".to_string()),))
        },
    )?;

    // remove-attribute - removes attribute
    linker.func_wrap(
        "tairitsu-browser:full/element@0.2.0",
        "remove-attribute",
        move |mut caller: wasmtime::Caller<'_, T>, self_handle: u64, name: String| {
            let state = get_state(caller.data_mut());
            if let Some(node) = state.dom.get_node_mut(self_handle) {
                node.remove_attribute(&name);
            }
            Ok(())
        },
    )?;

    // remove-attribute-ns - removes attribute (ignoring namespace)
    linker.func_wrap(
        "tairitsu-browser:full/element@0.2.0",
        "remove-attribute-ns",
        move |mut caller: wasmtime::Caller<'_, T>, self_handle: u64, _namespace: Option<String>, _local_name: String| {
            let state = get_state(caller.data_mut());
            if let Some(node) = state.dom.get_node_mut(self_handle) {
                node.remove_attribute(&_local_name);
            }
            Ok(())
        },
    )?;

    // toggle-attribute - returns false
    linker.func_wrap(
        "tairitsu-browser:full/element@0.2.0",
        "toggle-attribute",
        move |_caller: wasmtime::Caller<'_, T>, _self_handle: u64, _name: String, _force: Option<bool>| -> Result<bool, wasmtime::Error> {
            Ok(false)
        },
    )?;

    // has-attribute - checks if attribute exists
    linker.func_wrap(
        "tairitsu-browser:full/element@0.2.0",
        "has-attribute",
        move |mut caller: wasmtime::Caller<'_, T>, self_handle: u64, name: String| -> Result<bool, wasmtime::Error> {
            let state = get_state(caller.data_mut());
            if let Some(node) = state.dom.get_node(self_handle) {
                return Ok(node.get_attribute(&name).is_some());
            }
            Ok(false)
        },
    )?;

    // has-attribute-ns - returns false
    linker.func_wrap(
        "tairitsu-browser:full/element@0.2.0",
        "has-attribute-ns",
        move |_caller: wasmtime::Caller<'_, T>, _self_handle: u64, _namespace: Option<String>, _local_name: String>| -> Result<bool, wasmtime::Error> {
            Ok(false)
        },
    )?;

    // get-element-by-id (on element) - searches in subtree
    linker.func_wrap(
        "tairitsu-browser:full/element@0.2.0",
        "get-element-by-id",
        move |mut caller: wasmtime::Caller<'_, T>, _self_handle: u64, id: String| -> Result<Option<u64>, wasmtime::Error> {
            let state = get_state(caller.data_mut());
            Ok(state.dom.get_element_by_id(&id))
        },
    )?;

    // get-elements-by-class-name - returns empty list
    linker.func_wrap(
        "tairitsu-browser:full/element@0.2.0",
        "get-elements-by-class-name",
        move |_caller: wasmtime::Caller<'_, T>, _self_handle: u64, _class_names: String>| -> Result<u64, wasmtime::Error> {
            Ok(1) // Dummy HTMLCollection handle
        },
    )?;

    // get-elements-by-tag-name - returns empty list
    linker.func_wrap(
        "tairitsu-browser:full/element@0.2.0",
        "get-elements-by-tag-name",
        move |_caller: wasmtime::Caller<'_, T>, _self_handle: u64, _qualified_name: String>| -> Result<u64, wasmtime::Error> {
            Ok(1) // Dummy HTMLCollection handle
        },
    )?;

    // get-elements-by-tag-name-ns - returns empty list
    linker.func_wrap(
        "tairitsu-browser:full/element@0.2.0",
        "get-elements-by-tag-name-ns",
        move |_caller: wasmtime::Caller<'_, T>, _self_handle: u64, _namespace: Option<String>, _local_name: String>| -> Result<u64, wasmtime::Error> {
            Ok(1) // Dummy HTMLCollection handle
        },
    )?;

    // get-elements-by-name - returns empty list
    linker.func_wrap(
        "tairitsu-browser:full/element@0.2.0",
        "get-elements-by-name",
        move |_caller: wasmtime::Caller<'_, T>, _self_handle: u64, _name: String>| -> Result<u64, wasmtime::Error> {
            Ok(1) // Dummy NodeList handle
        },
    )?;

    // query-selector - delegates to DOM
    linker.func_wrap(
        "tairitsu-browser:full/element@0.2.0",
        "query-selector",
        move |mut caller: wasmtime::Caller<'_, T>, _self_handle: u64, selectors: String| -> Result<Option<u64>, wasmtime::Error> {
            let state = get_state(caller.data_mut());
            Ok(state.dom.query_selector(&selectors))
        },
    )?;

    // query-selector-all - returns dummy handle
    linker.func_wrap(
        "tairitsu-browser:full/element@0.2.0",
        "query-selector-all",
        move |_caller: wasmtime::Caller<'_, T>, _self_handle: u64, _selectors: String>| -> Result<u64, wasmtime::Error> {
            Ok(1) // Dummy NodeList handle
        },
    )?;

    // closest - returns None
    linker.func_wrap(
        "tairitsu-browser:full/element@0.2.0",
        "closest",
        move |_caller: wasmtime::Caller<'_, T>, _self_handle: u64, _selectors: String>| -> Result<Option<u64>, wasmtime::Error> {
            Ok(None)
        },
    )?;

    // insert-adjacent-element - returns None
    linker.func_wrap(
        "tairitsu-browser:full/element@0.2.0",
        "insert-adjacent-element",
        move |_caller: wasmtime::Caller<'_, T>, _self_handle: u64, _position: String, _element: u64>| -> Result<Option<u64>, wasmtime::Error> {
            Ok(None)
        },
    )?;

    Ok(())
}
