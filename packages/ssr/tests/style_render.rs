//! E2E tests for style rendering in SSR
//!
//! These tests verify that:
//! - Style properties are correctly stored
//! - Style attributes are serialized correctly
//! - Class attributes work properly

use tairitsu_ssr::SsrDom;

#[test]
fn test_set_style_property() {
    let mut dom = SsrDom::new();
    let div = dom.create_element("div", None);

    dom.get_node_mut(div)
        .unwrap()
        .set_style_property("color", "red");

    let node = dom.get_node(div).unwrap();
    assert_eq!(node.get_style_property("color"), Some("red"));
}

#[test]
fn test_set_multiple_style_properties() {
    let mut dom = SsrDom::new();
    let div = dom.create_element("div", None);

    dom.get_node_mut(div)
        .unwrap()
        .set_style_property("color", "red");
    dom.get_node_mut(div)
        .unwrap()
        .set_style_property("font-size", "14px");
    dom.get_node_mut(div)
        .unwrap()
        .set_style_property("background-color", "blue");

    let node = dom.get_node(div).unwrap();
    assert_eq!(node.get_style_property("color"), Some("red"));
    assert_eq!(node.get_style_property("font-size"), Some("14px"));
    assert_eq!(node.get_style_property("background-color"), Some("blue"));
}

#[test]
fn test_remove_style_property() {
    let mut dom = SsrDom::new();
    let div = dom.create_element("div", None);

    dom.get_node_mut(div)
        .unwrap()
        .set_style_property("color", "red");
    assert_eq!(
        dom.get_node(div).unwrap().get_style_property("color"),
        Some("red")
    );

    dom.get_node_mut(div)
        .unwrap()
        .remove_style_property("color");
    assert_eq!(dom.get_node(div).unwrap().get_style_property("color"), None);
}

#[test]
fn test_html_render_with_style() {
    let mut dom = SsrDom::new();
    let div = dom.create_element("div", None);

    dom.get_node_mut(div)
        .unwrap()
        .set_style_property("color", "red");
    dom.get_node_mut(div)
        .unwrap()
        .set_style_property("font-size", "14px");

    let html = render_html(&dom, div);
    assert_eq!(html, r#"<div style="color:red;font-size:14px"></div>"#);
}

#[test]
fn test_html_render_with_single_style() {
    let mut dom = SsrDom::new();
    let div = dom.create_element("div", None);

    dom.get_node_mut(div)
        .unwrap()
        .set_style_property("color", "red");

    let html = render_html(&dom, div);
    assert_eq!(html, r#"<div style="color:red"></div>"#);
}

#[test]
fn test_set_class() {
    let mut dom = SsrDom::new();
    let div = dom.create_element("div", None);

    dom.get_node_mut(div).unwrap().set_class("foo bar baz");

    let node = dom.get_node(div).unwrap();
    assert_eq!(node.class, "foo bar baz");
}

#[test]
fn test_html_render_with_class() {
    let mut dom = SsrDom::new();
    let div = dom.create_element("div", None);

    dom.get_node_mut(div).unwrap().set_class("container main");

    let html = render_html(&dom, div);
    assert_eq!(html, r#"<div class="container main"></div>"#);
}

#[test]
fn test_html_render_with_class_and_style() {
    let mut dom = SsrDom::new();
    let div = dom.create_element("div", None);

    dom.get_node_mut(div).unwrap().set_class("container");
    dom.get_node_mut(div)
        .unwrap()
        .set_style_property("color", "red");

    let html = render_html(&dom, div);
    assert_eq!(html, r#"<div class="container" style="color:red"></div>"#);
}

#[test]
fn test_html_render_with_all_attributes() {
    let mut dom = SsrDom::new();
    let div = dom.create_element("div", None);

    dom.get_node_mut(div).unwrap().set_attribute("id", "my-div");
    dom.get_node_mut(div).unwrap().set_class("container");
    dom.get_node_mut(div)
        .unwrap()
        .set_style_property("color", "red");
    dom.get_node_mut(div)
        .unwrap()
        .set_attribute("data-value", "123");

    let html = render_html(&dom, div);
    // Check that all attributes are present
    assert!(html.contains("id=\"my-div\""));
    assert!(html.contains("class=\"container\""));
    assert!(html.contains("style=\"color:red\""));
    assert!(html.contains("data-value=\"123\""));
}

#[test]
fn test_complex_css_values() {
    let mut dom = SsrDom::new();
    let div = dom.create_element("div", None);

    // Test various CSS value formats
    dom.get_node_mut(div)
        .unwrap()
        .set_style_property("background", "linear-gradient(to right, red, blue)");
    dom.get_node_mut(div)
        .unwrap()
        .set_style_property("box-shadow", "0 2px 4px rgba(0,0,0,0.1)");
    dom.get_node_mut(div).unwrap().set_style_property(
        "font-family",
        "\"Segoe UI\", Tahoma, Geneva, Verdana, sans-serif",
    );

    let html = render_html(&dom, div);
    assert!(html.contains("linear-gradient"));
    assert!(html.contains("box-shadow"));
    assert!(html.contains("font-family"));
}

#[test]
fn test_empty_style_doesnt_render() {
    let mut dom = SsrDom::new();
    let div = dom.create_element("div", None);

    // Don't set any style
    let html = render_html(&dom, div);
    assert!(!html.contains("style="));
}

#[test]
fn test_empty_class_doesnt_render() {
    let mut dom = SsrDom::new();
    let div = dom.create_element("div", None);

    // Don't set any class
    let html = render_html(&dom, div);
    assert!(!html.contains("class="));
}

#[test]
fn test_update_style_property() {
    let mut dom = SsrDom::new();
    let div = dom.create_element("div", None);

    dom.get_node_mut(div)
        .unwrap()
        .set_style_property("color", "red");
    assert_eq!(
        dom.get_node(div).unwrap().get_style_property("color"),
        Some("red")
    );

    // Update the same property
    dom.get_node_mut(div)
        .unwrap()
        .set_style_property("color", "blue");
    assert_eq!(
        dom.get_node(div).unwrap().get_style_property("color"),
        Some("blue")
    );

    // Should only have one color property
    let html = render_html(&dom, div);
    assert_eq!(html, r#"<div style="color:blue"></div>"#);
}

#[test]
fn test_update_class() {
    let mut dom = SsrDom::new();
    let div = dom.create_element("div", None);

    dom.get_node_mut(div).unwrap().set_class("foo");
    assert_eq!(dom.get_node(div).unwrap().class, "foo");

    dom.get_node_mut(div).unwrap().set_class("bar");
    assert_eq!(dom.get_node(div).unwrap().class, "bar");

    let html = render_html(&dom, div);
    assert_eq!(html, r#"<div class="bar"></div>"#);
}

#[test]
fn test_style_with_special_characters() {
    let mut dom = SsrDom::new();
    let div = dom.create_element("div", None);

    // CSS with special characters
    dom.get_node_mut(div)
        .unwrap()
        .set_style_property("content", "\"Hello\"");

    let html = render_html(&dom, div);
    // Style values are not HTML-escaped in the style attribute
    // (they're CSS-escaped, but that's different)
    assert!(html.contains("content:"));
}

// Helper function to render a node to HTML
fn render_html(dom: &SsrDom, handle: u64) -> String {
    let mut buf = String::new();
    dom.render_node(handle, &mut buf);
    buf
}
