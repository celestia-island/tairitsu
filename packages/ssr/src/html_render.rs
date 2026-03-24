//! HTML serialization for SSR
//!
//! This module converts the in-memory SsrDom into HTML strings.

use crate::virtual_dom::SsrDom;
use serde::{Deserialize, Serialize};

/// Configuration for rendering a complete HTML document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FullDocumentConfig {
    /// Page language (e.g., "zh-CN", "en-US")
    pub lang: String,
    /// Character set encoding (e.g., "UTF-8", "GBK")
    pub charset: String,
    /// Viewport meta content (e.g., "width=device-width, initial-scale=1.0")
    pub viewport_content: String,
    /// Page title
    pub title: String,
    /// CSS stylesheet links to include
    pub css_links: Vec<String>,
}

impl Default for FullDocumentConfig {
    fn default() -> Self {
        Self {
            lang: "zh-CN".to_string(),
            charset: "UTF-8".to_string(),
            viewport_content: "width=device-width, initial-scale=1.0".to_string(),
            title: String::new(),
            css_links: Vec::new(),
        }
    }
}

impl SsrDom {
    /// Render the body subtree to HTML
    pub fn render_body_html(&self) -> String {
        let mut buf = String::new();
        self.render_node(self.body_handle(), &mut buf);
        buf
    }

    /// Render the head subtree to HTML
    pub fn render_head_html(&self) -> String {
        let mut buf = String::new();
        self.render_node(self.head_handle(), &mut buf);
        buf
    }

    /// Render a complete HTML document with DOCTYPE, html, head, and body tags
    pub fn render_full_document_html(&self, config: &FullDocumentConfig) -> String {
        let mut buf = String::new();

        // DOCTYPE declaration
        buf.push_str("<!DOCTYPE html>\n");

        // Opening html tag with language attribute
        buf.push_str("<html lang=\"");
        buf.push_str(&config.lang);
        buf.push_str("\">\n");

        // Head section
        buf.push_str("<head>\n");

        // Meta charset
        buf.push_str("  <meta charset=\"");
        buf.push_str(&config.charset);
        buf.push_str("\">\n");

        // Viewport meta tag
        buf.push_str("  <meta name=\"viewport\" content=\"");
        buf.push_str(&config.viewport_content);
        buf.push_str("\">\n");

        // Page title
        if !config.title.is_empty() {
            buf.push_str("  <title>");
            html_escape_into(&mut buf, &config.title);
            buf.push_str("</title>\n");
        }

        // CSS links
        for css_link in &config.css_links {
            buf.push_str("  <link rel=\"stylesheet\" href=\"");
            buf.push_str(css_link);
            buf.push_str("\">\n");
        }

        // Render head content from the DOM
        let head_html = self.render_head_html();
        if !head_html.is_empty() && head_html != "<head></head>" {
            buf.push_str(&head_html);
            buf.push('\n');
        }

        buf.push_str("</head>\n");

        // Body section
        buf.push_str("<body>\n");

        // Render body content from the DOM
        let body_html = self.render_body_html();
        buf.push_str(&body_html);

        buf.push_str("\n</body>\n");

        // Closing html tag
        buf.push_str("</html>\n");

        buf
    }

    /// Render a specific node and its children to HTML
    pub fn render_node(&self, handle: u64, buf: &mut String) {
        let Some(node) = self.get_node(handle) else {
            return;
        };

        match &node.kind {
            crate::virtual_dom::SsrNodeKind::Element { tag, .. } => {
                self.render_element(node, tag, buf);
            }
            crate::virtual_dom::SsrNodeKind::Text { data } => {
                html_escape_into(buf, data);
            }
        }
    }

    fn render_element(&self, node: &crate::virtual_dom::SsrNode, tag: &str, buf: &mut String) {
        // Opening tag
        buf.push('<');
        buf.push_str(tag);

        // Attributes
        for (name, value) in &node.attributes {
            buf.push(' ');
            buf.push_str(name);
            buf.push_str("=\"");
            html_escape_attr_into(buf, value);
            buf.push('"');
        }

        // Class attribute
        if !node.class.is_empty() {
            buf.push_str(" class=\"");
            html_escape_attr_into(buf, &node.class);
            buf.push('"');
        }

        // Style attribute
        if !node.style_properties.is_empty() {
            buf.push_str(" style=\"");
            for (i, (prop, val)) in node.style_properties.iter().enumerate() {
                if i > 0 {
                    buf.push(';');
                }
                buf.push_str(prop);
                buf.push(':');
                buf.push_str(val);
            }
            buf.push('"');
        }

        buf.push('>');

        // Void elements don't have closing tags
        if is_void_element(tag) {
            return;
        }

        // Children
        for &child in &node.children {
            self.render_node(child, buf);
        }

        // Closing tag
        buf.push_str("</");
        buf.push_str(tag);
        buf.push('>');
    }
}

/// HTML-escape text content
fn html_escape_into(buf: &mut String, s: &str) {
    for ch in s.chars() {
        match ch {
            '&' => buf.push_str("&amp;"),
            '<' => buf.push_str("&lt;"),
            '>' => buf.push_str("&gt;"),
            _ => buf.push(ch),
        }
    }
}

/// HTML-escape an attribute value
fn html_escape_attr_into(buf: &mut String, s: &str) {
    for ch in s.chars() {
        match ch {
            '&' => buf.push_str("&amp;"),
            '"' => buf.push_str("&quot;"),
            '<' => buf.push_str("&lt;"),
            '>' => buf.push_str("&gt;"),
            _ => buf.push(ch),
        }
    }
}

/// Returns true for HTML void elements that must not have a closing tag
fn is_void_element(tag: &str) -> bool {
    matches!(
        tag,
        "area"
            | "base"
            | "br"
            | "col"
            | "embed"
            | "hr"
            | "img"
            | "input"
            | "link"
            | "meta"
            | "param"
            | "source"
            | "track"
            | "wbr"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_html_escape() {
        let mut buf = String::new();
        html_escape_into(&mut buf, "Hello <world> & \"friends\"");
        // Quotes don't need to be escaped in text content, only in attributes
        assert_eq!(buf, "Hello &lt;world&gt; &amp; \"friends\"");
    }

    #[test]
    fn test_html_attr_escape() {
        let mut buf = String::new();
        html_escape_attr_into(&mut buf, "a\"b&c");
        assert_eq!(buf, "a&quot;b&amp;c");
    }

    #[test]
    fn test_render_simple_element() {
        let mut dom = SsrDom::new();
        let div = dom.create_element("div", None);
        dom.get_node_mut(div).unwrap().set_attribute("id", "test");

        let mut buf = String::new();
        dom.render_node(div, &mut buf);
        assert_eq!(buf, r#"<div id="test"></div>"#);
    }

    #[test]
    fn test_render_with_class() {
        let mut dom = SsrDom::new();
        let div = dom.create_element("div", None);
        dom.get_node_mut(div).unwrap().set_class("foo bar");

        let mut buf = String::new();
        dom.render_node(div, &mut buf);
        assert_eq!(buf, r#"<div class="foo bar"></div>"#);
    }

    #[test]
    fn test_render_with_style() {
        let mut dom = SsrDom::new();
        let div = dom.create_element("div", None);
        dom.get_node_mut(div)
            .unwrap()
            .set_style_property("color", "red");
        dom.get_node_mut(div)
            .unwrap()
            .set_style_property("font-size", "14px");

        let mut buf = String::new();
        dom.render_node(div, &mut buf);
        assert_eq!(buf, r#"<div style="color:red;font-size:14px"></div>"#);
    }

    #[test]
    fn test_render_text_node() {
        let mut dom = SsrDom::new();
        let text = dom.create_text_node("Hello & world");

        let mut buf = String::new();
        dom.render_node(text, &mut buf);
        assert_eq!(buf, "Hello &amp; world");
    }

    #[test]
    fn test_render_nested() {
        let mut dom = SsrDom::new();
        let div = dom.create_element("div", None);
        let span = dom.create_element("span", None);
        let text = dom.create_text_node("Hello");

        dom.append_child(span, text).unwrap();
        dom.append_child(div, span).unwrap();

        let mut buf = String::new();
        dom.render_node(div, &mut buf);
        assert_eq!(buf, "<div><span>Hello</span></div>");
    }

    #[test]
    fn test_void_element() {
        let mut dom = SsrDom::new();
        let img = dom.create_element("img", None);
        dom.get_node_mut(img)
            .unwrap()
            .set_attribute("src", "test.png");

        let mut buf = String::new();
        dom.render_node(img, &mut buf);
        assert_eq!(buf, r#"<img src="test.png">"#);
    }
}
