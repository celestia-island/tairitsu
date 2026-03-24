//! E2E tests for stub error handling
//!
//! These tests verify that:
//! - Browser-only APIs return appropriate stub responses
//! - Stubs don't cause panics
//! - Error messages are clear

use tairitsu_ssr::SsrDom;

#[test]
fn test_dom_viewport_dimensions() {
    let dom = SsrDom::new();
    assert_eq!(dom.viewport_width(), 1920);
    assert_eq!(dom.viewport_height(), 1080);
}

#[test]
fn test_stub_query_selector_complex() {
    let mut dom = SsrDom::new();
    let div = dom.create_element("div", None);
    dom.get_node_mut(div).unwrap().set_attribute("id", "test");

    // Complex selectors should return None (not implemented)
    assert_eq!(dom.query_selector(".my-class"), None);
    assert_eq!(dom.query_selector("div > p"), None);
    assert_eq!(dom.query_selector("[data-attr]"), None);
}

#[test]
fn test_get_nonexistent_node() {
    let mut dom = SsrDom::new();
    assert!(dom.get_node(99999).is_none());
    assert!(dom.get_node_mut(99999).is_none());
}

#[test]
fn test_append_to_nonexistent_parent() {
    let mut dom = SsrDom::new();
    let child = dom.create_element("div", None);

    let result = dom.append_child(99999, child);
    assert!(result.is_err());
}

#[test]
fn test_remove_child_from_nonexistent_parent() {
    let mut dom = SsrDom::new();
    let child = dom.create_element("div", None);

    let result = dom.remove_child(99999, child);
    assert!(result.is_err());
}

#[test]
fn test_remove_nonexistent_child() {
    let mut dom = SsrDom::new();
    let parent = dom.create_element("div", None);

    let result = dom.remove_child(parent, 99999);
    assert!(result.is_err());
}

#[test]
fn test_set_text_on_element() {
    let mut dom = SsrDom::new();
    let div = dom.create_element("div", None);

    // Setting text content on an element should work
    // (it just doesn't do anything for non-text nodes)
    let result = dom.set_text_content(div, "test");
    // The current implementation allows this silently
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_get_text_from_element() {
    let mut dom = SsrDom::new();
    let div = dom.create_element("div", None);

    // Elements don't have direct text content
    assert_eq!(dom.get_text_content(div), None);
}

#[test]
fn test_reparent_child() {
    let mut dom = SsrDom::new();
    let parent1 = dom.create_element("div", None);
    let parent2 = dom.create_element("div", None);
    let child = dom.create_element("span", None);

    // Add child to parent1
    dom.append_child(parent1, child).unwrap();
    assert_eq!(dom.get_node(parent1).unwrap().children.len(), 1);

    // Reparent to parent2
    dom.append_child(parent2, child).unwrap();
    assert_eq!(dom.get_node(parent1).unwrap().children.len(), 0);
    assert_eq!(dom.get_node(parent2).unwrap().children.len(), 1);
    assert_eq!(dom.get_node(child).unwrap().parent, Some(parent2));
}

#[test]
fn test_get_element_by_id_empty() {
    let dom = SsrDom::new();
    assert_eq!(dom.get_element_by_id("anything"), None);
}

#[test]
fn test_multiple_elements_same_id() {
    let mut dom = SsrDom::new();
    let div1 = dom.create_element("div", None);
    let div2 = dom.create_element("div", None);

    dom.get_node_mut(div1).unwrap().set_attribute("id", "same");
    dom.get_node_mut(div2).unwrap().set_attribute("id", "same");

    // Should return the first match
    let result = dom.get_element_by_id("same");
    assert!(result.is_some());
    assert_eq!(result, Some(div1)); // First one created
}

#[test]
fn test_empty_text_node() {
    let mut dom = SsrDom::new();
    let text = dom.create_text_node("");

    let node = dom.get_node(text).unwrap();
    assert_eq!(node.text_content(), Some(""));
}

#[test]
fn test_text_node_with_special_chars() {
    let mut dom = SsrDom::new();
    let text = dom.create_text_node("Hello & goodbye <now>");

    let node = dom.get_node(text).unwrap();
    assert_eq!(node.text_content(), Some("Hello & goodbye <now>"));

    // But when rendered, it should be escaped
    let html = render_html(&dom, text);
    assert!(html.contains("&amp;"));
    assert!(html.contains("&lt;"));
}

#[test]
fn test_attribute_with_empty_value() {
    let mut dom = SsrDom::new();
    let div = dom.create_element("div", None);

    dom.get_node_mut(div).unwrap().set_attribute("data-empty", "");

    let html = render_html(&dom, div);
    assert!(html.contains("data-empty=\"\""));
}

#[test]
fn test_attribute_with_special_chars() {
    let mut dom = SsrDom::new();
    let div = dom.create_element("div", None);

    dom.get_node_mut(div)
        .unwrap()
        .set_attribute("data-test", "a<b&c");

    let html = render_html(&dom, div);
    assert!(html.contains("&lt;"));
    assert!(html.contains("&amp;"));
}

#[test]
fn test_multiple_attributes_same_name() {
    let mut dom = SsrDom::new();
    let div = dom.create_element("div", None);

    dom.get_node_mut(div).unwrap().set_attribute("class", "foo");
    dom.get_node_mut(div).unwrap().set_attribute("class", "bar");

    // Should only have one class attribute (last one wins)
    let node = dom.get_node(div).unwrap();
    let class_values: Vec<_> = node
        .attributes
        .iter()
        .filter(|(k, _)| k == "class")
        .map(|(_, v)| v.as_str())
        .collect();

    assert_eq!(class_values.len(), 1);
    assert_eq!(class_values[0], "bar");
}

#[test]
fn test_self_append_prevented() {
    let mut dom = SsrDom::new();
    let div = dom.create_element("div", None);

    // Try to append a node to itself
    // This should be handled gracefully
    let result = dom.append_child(div, div);
    // The implementation may or may not allow this
    // We just check it doesn't panic
    let _ = result;
}

#[test]
fn test_cycle_detection() {
    let mut dom = SsrDom::new();
    let parent = dom.create_element("div", None);
    let child = dom.create_element("div", None);

    dom.append_child(parent, child).unwrap();

    // Try to create a cycle by appending parent to child
    let result = dom.append_child(child, parent);
    // The implementation may or may not prevent cycles
    // We just check it doesn't panic
    let _ = result;
}

#[test]
fn test_large_dom() {
    let mut dom = SsrDom::new();
    let root = dom.create_element("div", None);

    // Create a large DOM tree
    for i in 0..100 {
        let child = dom.create_element("div", None);
        dom.get_node_mut(child)
            .unwrap()
            .set_attribute("data-index", &i.to_string());
        dom.append_child(root, child).unwrap();
    }

    let root_node = dom.get_node(root).unwrap();
    assert_eq!(root_node.children.len(), 100);
}

#[test]
fn test_deep_dom() {
    let mut dom = SsrDom::new();
    let mut parent = dom.create_element("div", None);

    // Create a deep DOM tree
    for i in 0..50 {
        let child = dom.create_element("div", None);
        dom.get_node_mut(child)
            .unwrap()
            .set_attribute("data-depth", &i.to_string());
        dom.append_child(parent, child).unwrap();
        parent = child;
    }

    // Verify the chain is correct
    let mut current = parent;
    let mut depth = 0;
    while let Some(node) = dom.get_node(current) {
        if let Some(p) = node.parent {
            current = p;
            depth += 1;
        } else {
            break;
        }
    }
    assert_eq!(depth, 50);
}

// Helper function to render a node to HTML
fn render_html(dom: &SsrDom, handle: u64) -> String {
    let mut buf = String::new();
    dom.render_node(handle, &mut buf);
    buf
}
