//! Server-side in-memory DOM implementation
//!
//! This module provides a simple DOM representation for SSR.
//! It stores nodes in a HashMap and uses u64 handles to reference them.

use std::collections::HashMap;

/// Server-side DOM node
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SsrNode {
    pub handle: u64,
    pub kind: SsrNodeKind,
    pub attributes: Vec<(String, String)>,
    pub style_properties: Vec<(String, String)>,
    pub class: String,
    pub children: Vec<u64>,
    pub parent: Option<u64>,
}

/// Node kind - either an element or text node
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SsrNodeKind {
    Element {
        tag: String,
        namespace: Option<String>,
    },
    Text {
        data: String,
    },
}

impl SsrNode {
    /// Create a new element node
    pub fn element(handle: u64, tag: impl Into<String>) -> Self {
        Self {
            handle,
            kind: SsrNodeKind::Element {
                tag: tag.into(),
                namespace: None,
            },
            attributes: Vec::new(),
            style_properties: Vec::new(),
            class: String::new(),
            children: Vec::new(),
            parent: None,
        }
    }

    /// Create a new element node with namespace
    pub fn element_ns(handle: u64, tag: impl Into<String>, ns: Option<String>) -> Self {
        Self {
            handle,
            kind: SsrNodeKind::Element {
                tag: tag.into(),
                namespace: ns,
            },
            attributes: Vec::new(),
            style_properties: Vec::new(),
            class: String::new(),
            children: Vec::new(),
            parent: None,
        }
    }

    /// Create a new text node
    pub fn text(handle: u64, data: impl Into<String>) -> Self {
        Self {
            handle,
            kind: SsrNodeKind::Text { data: data.into() },
            attributes: Vec::new(),
            style_properties: Vec::new(),
            class: String::new(),
            children: Vec::new(),
            parent: None,
        }
    }

    /// Get the tag name if this is an element
    pub fn tag_name(&self) -> Option<&str> {
        match &self.kind {
            SsrNodeKind::Element { tag, .. } => Some(tag),
            SsrNodeKind::Text { .. } => None,
        }
    }

    /// Get the text content if this is a text node
    pub fn text_content(&self) -> Option<&str> {
        match &self.kind {
            SsrNodeKind::Element { .. } => None,
            SsrNodeKind::Text { data } => Some(data),
        }
    }

    /// Set text content (for text nodes)
    pub fn set_text_content(&mut self, data: impl Into<String>) {
        if let SsrNodeKind::Text {
            data: ref mut existing,
        } = &mut self.kind
        {
            *existing = data.into();
        }
    }

    /// Set an attribute
    pub fn set_attribute(&mut self, name: impl Into<String>, value: impl Into<String>) {
        let name = name.into();
        let value = value.into();

        // Remove existing attribute with same name
        self.attributes.retain(|(n, _)| n != &name);
        self.attributes.push((name, value));
    }

    /// Get an attribute value
    pub fn get_attribute(&self, name: &str) -> Option<&str> {
        self.attributes
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, v)| v.as_str())
    }

    /// Remove an attribute
    pub fn remove_attribute(&mut self, name: &str) {
        self.attributes.retain(|(n, _)| n != name);
    }

    /// Set a style property
    pub fn set_style_property(&mut self, property: impl Into<String>, value: impl Into<String>) {
        let property = property.into();
        let value = value.into();

        // Remove existing style property with same name
        self.style_properties.retain(|(n, _)| n != &property);
        self.style_properties.push((property, value));
    }

    /// Get a style property value
    pub fn get_style_property(&self, property: &str) -> Option<&str> {
        self.style_properties
            .iter()
            .find(|(n, _)| n == property)
            .map(|(_, v)| v.as_str())
    }

    /// Remove a style property
    pub fn remove_style_property(&mut self, property: &str) {
        self.style_properties.retain(|(n, _)| n != property);
    }

    /// Set the class
    pub fn set_class(&mut self, class: impl Into<String>) {
        self.class = class.into();
    }

    /// Add a child
    pub fn add_child(&mut self, child_handle: u64) {
        self.children.push(child_handle);
    }

    /// Remove a child
    pub fn remove_child(&mut self, child_handle: u64) -> bool {
        let pos = self.children.iter().position(|&h| h == child_handle);
        if let Some(idx) = pos {
            self.children.remove(idx);
            true
        } else {
            false
        }
    }
}

/// Server-side DOM - manages all nodes
pub struct SsrDom {
    nodes: HashMap<u64, SsrNode>,
    next_handle: u64,
    body_handle: u64,
    head_handle: u64,
    document_handle: u64,
}

impl Default for SsrDom {
    fn default() -> Self {
        Self::new()
    }
}

impl SsrDom {
    /// Create a new SSR DOM with document, head, and body elements
    pub fn new() -> Self {
        let mut nodes = HashMap::new();
        let mut next_handle = 1;

        // Create document node (handle 0 is reserved for "null")
        let document_handle = next_handle;
        next_handle += 1;
        nodes.insert(
            document_handle,
            SsrNode::element(document_handle, "document"),
        );

        // Create html element
        let html_handle = next_handle;
        next_handle += 1;
        nodes.insert(html_handle, SsrNode::element(html_handle, "html"));

        // Create head element
        let head_handle = next_handle;
        next_handle += 1;
        nodes.insert(head_handle, SsrNode::element(head_handle, "head"));

        // Create body element
        let body_handle = next_handle;
        next_handle += 1;
        nodes.insert(body_handle, SsrNode::element(body_handle, "body"));

        // Set up hierarchy: html -> head, body
        if let Some(html_node) = nodes.get_mut(&html_handle) {
            html_node.add_child(head_handle);
            html_node.add_child(body_handle);
            if let Some(head_node) = nodes.get_mut(&head_handle) {
                head_node.parent = Some(html_handle);
            }
            if let Some(body_node) = nodes.get_mut(&body_handle) {
                body_node.parent = Some(html_handle);
            }
        }

        Self {
            nodes,
            next_handle,
            body_handle,
            head_handle,
            document_handle,
        }
    }

    /// Get the body element handle
    pub fn body_handle(&self) -> u64 {
        self.body_handle
    }

    /// Get the head element handle
    pub fn head_handle(&self) -> u64 {
        self.head_handle
    }

    /// Get the document element handle
    pub fn document_handle(&self) -> u64 {
        self.document_handle
    }

    /// Allocate a new handle
    fn allocate_handle(&mut self) -> u64 {
        let handle = self.next_handle;
        self.next_handle += 1;
        handle
    }

    /// Create a new element
    pub fn create_element(&mut self, tag: &str, namespace: Option<&str>) -> u64 {
        let handle = self.allocate_handle();
        let node = match namespace {
            Some(ns) => SsrNode::element_ns(handle, tag, Some(ns.to_string())),
            None => SsrNode::element(handle, tag),
        };
        self.nodes.insert(handle, node);
        handle
    }

    /// Create a new text node
    pub fn create_text_node(&mut self, data: &str) -> u64 {
        let handle = self.allocate_handle();
        let node = SsrNode::text(handle, data);
        self.nodes.insert(handle, node);
        handle
    }

    /// Get a node by handle
    pub fn get_node(&self, handle: u64) -> Option<&SsrNode> {
        self.nodes.get(&handle)
    }

    /// Get a mutable node by handle
    pub fn get_node_mut(&mut self, handle: u64) -> Option<&mut SsrNode> {
        self.nodes.get_mut(&handle)
    }

    /// Append a child to a parent
    pub fn append_child(&mut self, parent: u64, child: u64) -> Result<(), String> {
        // First, get the old parent if any
        let old_parent = self.nodes.get(&child).and_then(|n| n.parent);

        // Remove from existing parent (if different from new parent)
        if let Some(old_parent) = old_parent {
            if old_parent != parent {
                if let Some(old_parent_node) = self.nodes.get_mut(&old_parent) {
                    old_parent_node.remove_child(child);
                }
            }
        }

        // Add to new parent
        if let Some(parent_node) = self.nodes.get_mut(&parent) {
            parent_node.add_child(child);
        } else {
            return Err("Parent node not found".to_string());
        }

        // Update child's parent reference
        if let Some(child_node) = self.nodes.get_mut(&child) {
            child_node.parent = Some(parent);
        }

        Ok(())
    }

    /// Remove a child from a parent
    pub fn remove_child(&mut self, parent: u64, child: u64) -> Result<(), String> {
        let parent_node = self.nodes.get_mut(&parent).ok_or("Parent node not found")?;

        if !parent_node.remove_child(child) {
            return Err("Child not found in parent".to_string());
        }

        if let Some(child_node) = self.nodes.get_mut(&child) {
            child_node.parent = None;
        }

        Ok(())
    }

    /// Get element by ID
    pub fn get_element_by_id(&self, id: &str) -> Option<u64> {
        for (&handle, node) in &self.nodes {
            if let Some(attr_value) = node.get_attribute("id") {
                if attr_value == id {
                    return Some(handle);
                }
            }
        }
        None
    }

    /// Query selector (basic implementation - only supports ID selector for now)
    pub fn query_selector(&self, selector: &str) -> Option<u64> {
        let selector = selector.trim();

        // ID selector: #id
        if let Some(id) = selector.strip_prefix('#') {
            return self.get_element_by_id(id);
        }

        // Tag selector: tagname
        if !selector.contains(|c: char| c.is_whitespace() || c == '.' || c == '#') {
            for (&handle, node) in &self.nodes {
                if node.tag_name() == Some(selector) {
                    return Some(handle);
                }
            }
        }

        None
    }

    /// Get inner text content of a node (for text nodes, returns the text)
    pub fn get_text_content(&self, handle: u64) -> Option<String> {
        let node = self.nodes.get(&handle)?;
        if let Some(text) = node.text_content() {
            return Some(text.to_string());
        }
        None
    }

    /// Set text content of a text node
    pub fn set_text_content(&mut self, handle: u64, text: &str) -> Result<(), String> {
        let node = self.nodes.get_mut(&handle).ok_or("Node not found")?;
        node.set_text_content(text);
        Ok(())
    }

    /// Iterate over all nodes
    pub fn iter_nodes(&self) -> impl Iterator<Item = (u64, &SsrNode)> {
        self.nodes.iter().map(|(&h, n)| (h, n))
    }

    /// Get the viewport width configuration
    pub fn viewport_width(&self) -> i32 {
        // Default viewport width - can be made configurable later
        1920
    }

    /// Get the viewport height configuration
    pub fn viewport_height(&self) -> i32 {
        // Default viewport height - can be made configurable later
        1080
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_dom() {
        let dom = SsrDom::new();
        assert_ne!(dom.body_handle(), 0);
        assert_ne!(dom.head_handle(), 0);
        assert_ne!(dom.document_handle(), 0);
    }

    #[test]
    fn test_create_element() {
        let mut dom = SsrDom::new();
        let div = dom.create_element("div", None);
        let node = dom.get_node(div).unwrap();
        assert_eq!(node.tag_name(), Some("div"));
    }

    #[test]
    fn test_create_text_node() {
        let mut dom = SsrDom::new();
        let text = dom.create_text_node("Hello");
        let node = dom.get_node(text).unwrap();
        assert_eq!(node.text_content(), Some("Hello"));
    }

    #[test]
    fn test_append_child() {
        let mut dom = SsrDom::new();
        let div = dom.create_element("div", None);
        let span = dom.create_element("span", None);

        dom.append_child(div, span).unwrap();

        let div_node = dom.get_node(div).unwrap();
        assert_eq!(div_node.children, vec![span]);

        let span_node = dom.get_node(span).unwrap();
        assert_eq!(span_node.parent, Some(div));
    }

    #[test]
    fn test_set_attribute() {
        let mut dom = SsrDom::new();
        let div = dom.create_element("div", None);
        dom.get_node_mut(div)
            .unwrap()
            .set_attribute("id", "test-id");

        let node = dom.get_node(div).unwrap();
        assert_eq!(node.get_attribute("id"), Some("test-id"));
    }

    #[test]
    fn test_get_element_by_id() {
        let mut dom = SsrDom::new();
        let div = dom.create_element("div", None);
        dom.get_node_mut(div).unwrap().set_attribute("id", "my-div");

        assert_eq!(dom.get_element_by_id("my-div"), Some(div));
        assert_eq!(dom.get_element_by_id("not-found"), None);
    }

    #[test]
    fn test_query_selector_id() {
        let mut dom = SsrDom::new();
        let div = dom.create_element("div", None);
        dom.get_node_mut(div).unwrap().set_attribute("id", "my-div");

        assert_eq!(dom.query_selector("#my-div"), Some(div));
        assert_eq!(dom.query_selector("#not-found"), None);
    }
}
