//! E2E tests for basic SSR rendering functionality
//!
//! These tests verify that the SSR system can:
//! - Create DOM elements
//! - Set attributes
//! - Build nested structures
//! - Serialize to correct HTML

use tairitsu_ssr::{SsrConfig, SsrDom};

#[test]
fn test_dom_creation() {
    let dom = SsrDom::new();
    assert_ne!(dom.body_handle(), 0, "body handle should be non-zero");
    assert_ne!(dom.head_handle(), 0, "head handle should be non-zero");
    assert_ne!(dom.document_handle(), 0, "document handle should be non-zero");
}

#[test]
fn test_create_element() {
    let mut dom = SsrDom::new();
    let div = dom.create_element("div", None);

    let node = dom.get_node(div).expect("div node should exist");
    assert_eq!(node.tag_name(), Some("div"));
}

#[test]
fn test_create_element_with_namespace() {
    let mut dom = SsrDom::new();
    let svg = dom.create_element("svg", Some("http://www.w3.org/2000/svg"));

    let node = dom.get_node(svg).expect("svg node should exist");
    assert_eq!(node.tag_name(), Some("svg"));
}

#[test]
fn test_create_text_node() {
    let mut dom = SsrDom::new();
    let text = dom.create_text_node("Hello, World!");

    let node = dom.get_node(text).expect("text node should exist");
    assert_eq!(node.text_content(), Some("Hello, World!"));
}

#[test]
fn test_set_attribute() {
    let mut dom = SsrDom::new();
    let div = dom.create_element("div", None);

    dom.get_node_mut(div)
        .unwrap()
        .set_attribute("id", "test-id");
    dom.get_node_mut(div)
        .unwrap()
        .set_attribute("class", "foo bar");

    let node = dom.get_node(div).unwrap();
    assert_eq!(node.get_attribute("id"), Some("test-id"));
    assert_eq!(node.get_attribute("class"), Some("foo bar"));
}

#[test]
fn test_remove_attribute() {
    let mut dom = SsrDom::new();
    let div = dom.create_element("div", None);

    dom.get_node_mut(div)
        .unwrap()
        .set_attribute("id", "test-id");
    assert_eq!(dom.get_node(div).unwrap().get_attribute("id"), Some("test-id"));

    dom.get_node_mut(div).unwrap().remove_attribute("id");
    assert_eq!(dom.get_node(div).unwrap().get_attribute("id"), None);
}

#[test]
fn test_append_child() {
    let mut dom = SsrDom::new();
    let parent = dom.create_element("div", None);
    let child = dom.create_element("span", None);

    dom.append_child(parent, child).unwrap();

    let parent_node = dom.get_node(parent).unwrap();
    assert_eq!(parent_node.children, vec![child]);

    let child_node = dom.get_node(child).unwrap();
    assert_eq!(child_node.parent, Some(parent));
}

#[test]
fn test_remove_child() {
    let mut dom = SsrDom::new();
    let parent = dom.create_element("div", None);
    let child = dom.create_element("span", None);

    dom.append_child(parent, child).unwrap();
    assert_eq!(dom.get_node(parent).unwrap().children.len(), 1);

    dom.remove_child(parent, child).unwrap();
    assert_eq!(dom.get_node(parent).unwrap().children.len(), 0);
    assert_eq!(dom.get_node(child).unwrap().parent, None);
}

#[test]
fn test_nested_structure() {
    let mut dom = SsrDom::new();
    let div = dom.create_element("div", None);
    let p1 = dom.create_element("p", None);
    let p2 = dom.create_element("p", None);
    let text1 = dom.create_text_node("First paragraph");
    let text2 = dom.create_text_node("Second paragraph");

    dom.append_child(p1, text1).unwrap();
    dom.append_child(p2, text2).unwrap();
    dom.append_child(div, p1).unwrap();
    dom.append_child(div, p2).unwrap();

    let div_node = dom.get_node(div).unwrap();
    assert_eq!(div_node.children.len(), 2);
}

#[test]
fn test_get_element_by_id() {
    let mut dom = SsrDom::new();
    let div1 = dom.create_element("div", None);
    let div2 = dom.create_element("div", None);

    dom.get_node_mut(div1).unwrap().set_attribute("id", "first");
    dom.get_node_mut(div2).unwrap().set_attribute("id", "second");

    assert_eq!(dom.get_element_by_id("first"), Some(div1));
    assert_eq!(dom.get_element_by_id("second"), Some(div2));
    assert_eq!(dom.get_element_by_id("nonexistent"), None);
}

#[test]
fn test_query_selector_id() {
    let mut dom = SsrDom::new();
    let div = dom.create_element("div", None);
    dom.get_node_mut(div).unwrap().set_attribute("id", "my-div");

    assert_eq!(dom.query_selector("#my-div"), Some(div));
    assert_eq!(dom.query_selector("#not-found"), None);
}

#[test]
fn test_query_selector_tag() {
    let mut dom = SsrDom::new();
    let _div = dom.create_element("div", None);

    // Tag selector should find first matching element
    let result = dom.query_selector("div");
    assert!(result.is_some());
}

#[test]
fn test_html_render_simple() {
    let mut dom = SsrDom::new();
    let div = dom.create_element("div", None);
    dom.get_node_mut(div)
        .unwrap()
        .set_attribute("id", "test");

    let html = dom.render_node_html(div);
    assert_eq!(html, r#"<div id="test"></div>"#);
}

#[test]
fn test_html_render_with_text() {
    let mut dom = SsrDom::new();
    let p = dom.create_element("p", None);
    let text = dom.create_text_node("Hello, World!");

    dom.append_child(p, text).unwrap();

    let html = dom.render_node_html(p);
    assert_eq!(html, "<p>Hello, World!</p>");
}

#[test]
fn test_html_render_nested() {
    let mut dom = SsrDom::new();
    let div = dom.create_element("div", None);
    let p = dom.create_element("p", None);
    let text = dom.create_text_node("Content");

    dom.append_child(p, text).unwrap();
    dom.append_child(div, p).unwrap();

    let html = dom.render_node_html(div);
    assert_eq!(html, "<div><p>Content</p></div>");
}

#[test]
fn test_html_render_multiple_children() {
    let mut dom = SsrDom::new();
    let ul = dom.create_element("ul", None);
    let li1 = dom.create_element("li", None);
    let li2 = dom.create_element("li", None);
    let text1 = dom.create_text_node("Item 1");
    let text2 = dom.create_text_node("Item 2");

    dom.append_child(li1, text1).unwrap();
    dom.append_child(li2, text2).unwrap();
    dom.append_child(ul, li1).unwrap();
    dom.append_child(ul, li2).unwrap();

    let html = dom.render_node_html(ul);
    assert_eq!(html, "<ul><li>Item 1</li><li>Item 2</li></ul>");
}

#[test]
fn test_html_escape() {
    let mut dom = SsrDom::new();
    let div = dom.create_element("div", None);
    let text = dom.create_text_node("<script>alert('XSS')</script>");

    dom.append_child(div, text).unwrap();

    let html = dom.render_node_html(div);
    assert!(!html.contains("<script>"));
    assert!(html.contains("&lt;script&gt;"));
}

#[test]
fn test_attribute_escape() {
    let mut dom = SsrDom::new();
    let div = dom.create_element("div", None);
    dom.get_node_mut(div)
        .unwrap()
        .set_attribute("title", "Hello \"World\"");

    let html = dom.render_node_html(div);
    assert!(html.contains("&quot;"));
    assert!(!html.contains("\"\""));
}

#[test]
fn test_void_elements() {
    let mut dom = SsrDom::new();

    // Test various void elements
    for (tag, expected) in [
        ("img", r#"<img src="test.png">"#),
        ("input", r#"<input type="text">"#),
        ("br", "<br>"),
        ("hr", "<hr>"),
    ] {
        let el = dom.create_element(tag, None);
        if tag == "img" {
            dom.get_node_mut(el).unwrap().set_attribute("src", "test.png");
        } else if tag == "input" {
            dom.get_node_mut(el).unwrap().set_attribute("type", "text");
        }
        let html = dom.render_node_html(el);
        assert_eq!(html, expected, "Void element {} should render without closing tag", tag);
    }
}

#[test]
fn test_text_content_get() {
    let mut dom = SsrDom::new();
    let text = dom.create_text_node("Hello");

    assert_eq!(dom.get_text_content(text), Some("Hello".to_string()));
}

#[test]
fn test_text_content_set() {
    let mut dom = SsrDom::new();
    let text = dom.create_text_node("Old");

    dom.set_text_content(text, "New").unwrap();
    assert_eq!(dom.get_text_content(text), Some("New".to_string()));
}

#[test]
fn test_config_default() {
    let config = SsrConfig::default();
    assert_eq!(config.viewport_width, 1920);
    assert_eq!(config.viewport_height, 1080);
}

#[test]
fn test_config_new() {
    let config = SsrConfig::new(1280, 720);
    assert_eq!(config.viewport_width, 1280);
    assert_eq!(config.viewport_height, 720);
}

// Helper extension for render_node_html
trait SsrDomExt {
    fn render_node_html(&self, handle: u64) -> String;
}

impl SsrDomExt for SsrDom {
    fn render_node_html(&self, handle: u64) -> String {
        let mut buf = String::new();
        self.render_node(handle, &mut buf);
        buf
    }
}
