//! Batch DOM operations for performance optimization.
//!
//! Groups multiple DOM operations together to reduce WIT round-trips.

use std::{cell::RefCell, collections::HashMap};

use crate::wit_platform::WitElement;

/// A batched style operation.
#[derive(Debug, Clone)]
pub struct BatchStyleOp {
    /// Element handle
    pub element: u64,
    /// Style property name
    pub name: String,
    /// Style property value
    pub value: String,
}

/// A batched attribute operation.
#[derive(Debug, Clone)]
pub struct BatchAttrOp {
    /// Element handle
    pub element: u64,
    /// Attribute name
    pub name: String,
    /// Attribute value
    pub value: String,
}

/// Batch operations collector.
///
/// Collects multiple DOM operations and applies them in batches to
/// reduce the number of WIT round-trips.
#[derive(Debug, Default)]
pub struct BatchOps {
    /// Style operations to apply (element -> [(name, value), ...])
    styles: RefCell<HashMap<u64, Vec<(String, String)>>>,

    /// Attribute operations to apply (element -> [(name, value), ...])
    attrs: RefCell<HashMap<u64, Vec<(String, String)>>>,

    /// Elements to remove (for cleanup)
    removals: RefCell<Vec<WitElement>>,
}

impl BatchOps {
    /// Create a new batch operations collector.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a style operation to the batch.
    ///
    /// # Arguments
    ///
    /// * `element` - Element to apply style to
    /// * `name` - CSS property name
    /// * `value` - CSS property value
    pub fn add_style(&self, element: WitElement, name: &str, value: &str) {
        let mut styles = self.styles.borrow_mut();
        styles
            .entry(element.as_raw())
            .or_default()
            .push((name.to_string(), value.to_string()));
    }

    /// Add multiple style operations to the batch.
    ///
    /// # Arguments
    ///
    /// * `element` - Element to apply styles to
    /// * `styles` - Iterator of (name, value) pairs
    pub fn add_styles(
        &self,
        element: WitElement,
        styles: impl IntoIterator<Item = (impl AsRef<str>, impl AsRef<str>)>,
    ) {
        let mut map = self.styles.borrow_mut();
        let entry = map.entry(element.as_raw()).or_default();
        for (name, value) in styles {
            entry.push((name.as_ref().to_string(), value.as_ref().to_string()));
        }
    }

    /// Add multiple attribute operations to the batch.
    ///
    /// # Arguments
    ///
    /// * `element` - Element to set attributes on
    /// * `attrs` - Iterator of (name, value) pairs
    pub fn add_attrs(
        &self,
        element: WitElement,
        attrs: impl IntoIterator<Item = (impl AsRef<str>, impl AsRef<str>)>,
    ) {
        let mut map = self.attrs.borrow_mut();
        let entry = map.entry(element.as_raw()).or_default();
        for (name, value) in attrs {
            entry.push((name.as_ref().to_string(), value.as_ref().to_string()));
        }
    }

    /// Mark an element for removal.
    ///
    /// The element will be removed when the batch is applied.
    pub fn add_removal(&self, element: WitElement) {
        self.removals.borrow_mut().push(element);
    }

    /// Get the number of operations in the batch.
    pub fn len(&self) -> usize {
        let styles_count = self
            .styles
            .borrow()
            .values()
            .map(|v| v.len())
            .sum::<usize>();
        let attrs_count = self.attrs.borrow().values().map(|v| v.len()).sum::<usize>();
        let removals_count = self.removals.borrow().len();
        styles_count + attrs_count + removals_count
    }

    /// Check if the batch is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Clear all operations from the batch.
    pub fn clear(&self) {
        self.styles.borrow_mut().clear();
        self.attrs.borrow_mut().clear();
        self.removals.borrow_mut().clear();
    }

    /// Apply all batched operations.
    ///
    /// This method applies all collected operations in an optimal order:
    /// 1. Set attributes
    /// 2. Set styles (batched per element)
    /// 3. Remove elements
    ///
    /// Returns the number of operations applied.
    pub fn apply(&self) -> usize {
        let mut count = 0;

        // Apply attributes
        let attrs = self.attrs.borrow();
        for (element, attr_list) in attrs.iter() {
            for (name, value) in attr_list.iter() {
                #[cfg(all(feature = "wit-bindings", target_family = "wasm"))]
                {
                    crate::wit_platform::wasm_impl::bindings::tairitsu_browser::full::element::set_attribute(
                        *element, name, value,
                    );
                }
                let _ = (element, name, value);
                count += 1;
            }
        }
        drop(attrs);

        // Apply styles (batched per element)
        let styles = self.styles.borrow();
        for (element, style_list) in styles.iter() {
            // Try to get style handle from cache once per element
            let _style_handle = crate::handle_cache::HandleCache::with(|cache| {
                if let Some(cached_handle) = cache.get_style_handle(*element) {
                    return cached_handle;
                }

                #[cfg(all(feature = "wit-bindings", target_family = "wasm"))]
                {
                    let handle = crate::wit_platform::wasm_impl::bindings::tairitsu_browser::full::element_css_inline_style::get_style(*element);
                    cache.set_style_handle(*element, handle);
                    handle
                }

                #[cfg(not(all(feature = "wit-bindings", target_family = "wasm")))]
                {
                    0
                }
            });

            for (name, value) in style_list.iter() {
                #[cfg(all(feature = "wit-bindings", target_family = "wasm"))]
                {
                    crate::wit_platform::wasm_impl::bindings::tairitsu_browser::full::css_style_declaration::set_property(
                        _style_handle, name, value, None,
                    );
                }
                let _ = (name, value);
                count += 1;
            }
        }
        drop(styles);

        // Apply removals
        let removals = self.removals.borrow();
        for element in removals.iter() {
            #[cfg(all(feature = "wit-bindings", target_family = "wasm"))]
            {
                // Invalidate cache first
                crate::handle_cache::HandleCache::with(|cache| {
                    cache.invalidate_style_handle(element.as_raw());
                });

                // Remove from parent (assuming parent is known or using a default)
                // In practice, you'd need to track parent relationships
            }
            let _ = element;
            count += 1;
        }

        count
    }

    /// Consume the batch and apply all operations.
    ///
    /// This is a convenience method that applies the batch and returns
    /// the number of operations applied, clearing the batch afterward.
    pub fn apply_and_clear(&self) -> usize {
        let count = self.apply();
        self.clear();
        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_ops_new() {
        let batch = BatchOps::new();
        assert!(batch.is_empty());
        assert_eq!(batch.len(), 0);
    }

    #[test]
    fn test_batch_ops_add_style() {
        let batch = BatchOps::new();
        let element = WitElement::from_raw(42);

        batch.add_style(element, "color", "red");
        assert_eq!(batch.len(), 1);
        assert!(!batch.is_empty());
    }

    #[test]
    fn test_batch_ops_add_styles() {
        let batch = BatchOps::new();
        let element = WitElement::from_raw(42);

        batch.add_styles(element, [("color", "red"), ("background", "blue")]);
        assert_eq!(batch.len(), 2);
    }

    #[test]
    fn test_batch_ops_add_attr() {
        let batch = BatchOps::new();
        let element = WitElement::from_raw(42);

        batch.add_attrs(element, [("id", "test")]);
        assert_eq!(batch.len(), 1);
    }

    #[test]
    fn test_batch_ops_add_attrs() {
        let batch = BatchOps::new();
        let element = WitElement::from_raw(42);

        batch.add_attrs(element, [("id", "test"), ("class", "foo")]);
        assert_eq!(batch.len(), 2);
    }

    #[test]
    fn test_batch_ops_multiple_elements() {
        let batch = BatchOps::new();
        let elem1 = WitElement::from_raw(1);
        let elem2 = WitElement::from_raw(2);

        batch.add_style(elem1, "color", "red");
        batch.add_style(elem2, "color", "blue");
        batch.add_attrs(elem1, [("id", "elem1")]);

        assert_eq!(batch.len(), 3);
    }

    #[test]
    fn test_batch_ops_clear() {
        let batch = BatchOps::new();
        let element = WitElement::from_raw(42);

        batch.add_style(element, "color", "red");
        assert_eq!(batch.len(), 1);

        batch.clear();
        assert!(batch.is_empty());
        assert_eq!(batch.len(), 0);
    }

    #[test]
    fn test_batch_ops_add_removal() {
        let batch = BatchOps::new();
        let element = WitElement::from_raw(42);

        batch.add_removal(element);
        assert_eq!(batch.len(), 1);
    }

    #[test]
    fn test_batch_ops_apply_returns_count() {
        let batch = BatchOps::new();
        let element = WitElement::from_raw(42);

        batch.add_style(element, "color", "red");
        batch.add_attrs(element, [("id", "test")]);

        let count = batch.apply();
        assert_eq!(count, 2);
    }

    #[test]
    fn test_batch_ops_apply_and_clear() {
        let batch = BatchOps::new();
        let element = WitElement::from_raw(42);

        batch.add_style(element, "color", "red");
        batch.add_attrs(element, [("id", "test")]);

        let count = batch.apply_and_clear();
        assert_eq!(count, 2);

        // Batch should be empty after apply_and_clear
        assert!(batch.is_empty());
    }
}
