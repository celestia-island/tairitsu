//! Trie-based route matching structure.

use std::{cell::RefCell, collections::HashMap, sync::Arc};

use tairitsu_vdom::VNode;

use super::{Params, RouteHandler, RouteSegment};

/// A node in the route matching trie
pub struct TrieNode {
    /// Child nodes keyed by segment
    children: RefCell<HashMap<String, Arc<TrieNode>>>,
    /// Dynamic child (for :param segments)
    dynamic_child: RefCell<Option<Arc<TrieNode>>>,
    /// Wildcard child (for * segments)
    wildcard_child: RefCell<Option<Arc<TrieNode>>>,
    /// Handler at this node (if any)
    handler: RefCell<Option<RouteHandler>>,
    /// Name of the route (if any)
    name: RefCell<Option<String>>,
}

impl TrieNode {
    /// Create a new trie node
    pub fn new() -> Self {
        Self {
            children: RefCell::new(HashMap::new()),
            dynamic_child: RefCell::new(None),
            wildcard_child: RefCell::new(None),
            handler: RefCell::new(None),
            name: RefCell::new(None),
        }
    }

    /// Insert a route into the trie
    pub fn insert(&self, segments: Vec<RouteSegment>, handler: RouteHandler) {
        self.insert_named(segments, None, handler);
    }

    /// Insert a named route into the trie
    pub fn insert_named(
        &self,
        segments: Vec<RouteSegment>,
        name: Option<String>,
        handler: RouteHandler,
    ) {
        if segments.is_empty() {
            *self.handler.borrow_mut() = Some(handler);
            if let Some(n) = name {
                *self.name.borrow_mut() = Some(n);
            }
            return;
        }

        let first = &segments[0];
        let rest = &segments[1..];

        match first.segment_type() {
            super::SegmentType::Static => {
                let key = first.to_string();
                let mut children = self.children.borrow_mut();
                let child = children
                    .entry(key)
                    .or_insert_with(|| Arc::new(TrieNode::new()))
                    .clone();
                drop(children);
                child.insert_named(rest.to_vec(), name, handler);
            }
            super::SegmentType::Dynamic => {
                let mut dynamic_child = self.dynamic_child.borrow_mut();
                let child = dynamic_child
                    .get_or_insert_with(|| Arc::new(TrieNode::new()))
                    .clone();
                drop(dynamic_child);
                child.insert_named(rest.to_vec(), name, handler);
            }
            super::SegmentType::Wildcard => {
                let mut wildcard_child = self.wildcard_child.borrow_mut();
                let child = wildcard_child
                    .get_or_insert_with(|| Arc::new(TrieNode::new()))
                    .clone();
                drop(wildcard_child);
                child.insert_named(rest.to_vec(), name, handler);
            }
        }
    }

    /// Find a route matching the given segments
    pub fn find(&self, segments: &[RouteSegment]) -> Option<(RouteHandler, Params, Option<String>)> {
        self.find_helper(segments, &mut Params::new())
    }

    fn find_helper(
        &self,
        segments: &[RouteSegment],
        params: &mut Params,
    ) -> Option<(RouteHandler, Params, Option<String>)> {
        if segments.is_empty() {
            let handler_guard = self.handler.borrow();
            if let Some(handler) = handler_guard.as_ref() {
                return Some((handler.clone(), params.clone(), self.name.borrow().clone()));
            }
            return None;
        }

        let first = &segments[0];
        let rest = &segments[1..];

        // Try static match first
        {
            let children = self.children.borrow();
            if let Some(child) = children.get(&first.to_string()) {
                if let Some(result) = child.find_helper(rest, params) {
                    return Some(result);
                }
            }
        }

        // Try dynamic match
        {
            let dynamic_child = self.dynamic_child.borrow();
            if let Some(child) = dynamic_child.as_ref() {
                if let Some(param_name) = first.param_name() {
                    params.insert(param_name.to_string(), first.to_string());
                    if let Some(result) = child.find_helper(rest, params) {
                        return Some(result);
                    }
                    params.remove(param_name);
                }
            }
        }

        // Try wildcard match
        {
            let wildcard_child = self.wildcard_child.borrow();
            if let Some(child) = wildcard_child.as_ref() {
                if let Some(result) = child.find_helper(rest, params) {
                    return Some(result);
                }
            }
        }

        None
    }

    /// Find a prefix match (for nested routes)
    pub fn find_prefix(
        &self,
        segments: &[RouteSegment],
    ) -> Option<(RouteHandler, Params, Option<String>)> {
        self.find_prefix_helper(segments, &mut Params::new(), 0)
    }

    fn find_prefix_helper(
        &self,
        segments: &[RouteSegment],
        params: &mut Params,
        depth: usize,
    ) -> Option<(RouteHandler, Params, Option<String>)> {
        // Check if we have a handler at this level
        {
            let handler_guard = self.handler.borrow();
            if let Some(handler) = handler_guard.as_ref() {
                return Some((
                    handler.clone(),
                    params.clone(),
                    self.name.borrow().clone(),
                ));
            }
        }

        if segments.is_empty() {
            return None;
        }

        let first = &segments[0];
        let rest = &segments[1..];

        // Try static match first
        {
            let children = self.children.borrow();
            if let Some(child) = children.get(&first.to_string()) {
                if let Some(result) = child.find_prefix_helper(rest, params, depth + 1) {
                    return Some(result);
                }
            }
        }

        // Try dynamic match
        {
            let dynamic_child = self.dynamic_child.borrow();
            if let Some(child) = dynamic_child.as_ref() {
                if let Some(param_name) = first.param_name() {
                    params.insert(param_name.to_string(), first.to_string());
                    if let Some(result) = child.find_prefix_helper(rest, params, depth + 1) {
                        return Some(result);
                    }
                    params.remove(param_name);
                }
            }
        }

        // Try wildcard match
        {
            let wildcard_child = self.wildcard_child.borrow();
            if let Some(child) = wildcard_child.as_ref() {
                if let Some(result) = child.find_prefix_helper(rest, params, depth + 1) {
                    return Some(result);
                }
            }
        }

        None
    }

    /// Generate a URL for a named route
    pub fn url_for(&self, name: &str, params: &[(&str, &str)]) -> Option<String> {
        self.url_for_helper(name, params, &mut Vec::new())
    }

    fn url_for_helper(
        &self,
        name: &str,
        params: &[(&str, &str)],
        segments: &mut Vec<String>,
    ) -> Option<String> {
        // Check if this node has the name we're looking for
        if self.name.borrow().as_deref() == Some(name) {
            // Build the URL from collected segments
            let mut url = if segments.is_empty() {
                "/".to_string()
            } else {
                format!("/{}", segments.join("/"))
            };
            return Some(url);
        }

        // Search children
        let children = self.children.borrow();
        for (key, child) in children.iter() {
            let mut new_segments = segments.clone();
            new_segments.push(key.clone());
            if let Some(url) = child.url_for_helper(name, params, &mut new_segments) {
                return Some(url);
            }
        }

        None
    }
}

impl Default for TrieNode {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_handler(_params: Params) -> VNode {
        VNode::Text(tairitsu_vdom::VText::new("mock"))
    }

    #[test]
    fn test_trie_creation() {
        let node = TrieNode::new();
        assert!(node.children.is_empty());
        assert!(node.dynamic_child.is_none());
        assert!(node.wildcard_child.is_none());
    }

    #[test]
    fn test_find_empty() {
        let node = TrieNode::new();
        let segments = RouteSegment::parse_path("/test");
        assert!(node.find(&segments).is_none());
    }

    #[test]
    fn test_find_prefix_empty() {
        let node = TrieNode::new();
        let segments = RouteSegment::parse_path("/test");
        assert!(node.find_prefix(&segments).is_none());
    }
}
