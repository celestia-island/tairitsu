//! DOM element reference hook.
//!
//! This module provides [`use_element_ref`] for creating references to DOM elements
//! that can be accessed from hooks and event handlers. Unlike the generic [`use_ref`],
//! `ElementRef` is specifically designed to hold platform DOM element handles that
//! are automatically populated during the rendering/mounting process.
//!
//! # Platform differences
//!
//! On wasm32, the platform stores [`WitElement`](tairitsu_web::wit_platform::WitElement)
//! into element refs at mount time. On non-wasm targets, raw `u64` may be used.
//!
//! Use [`tairitsu_vdom::resolve_element_ref()`] to safely extract a
//! [`DomHandle`](tairitsu_vdom::DomHandle) regardless of platform, or use
//! [`crate::use_dom_ref`] for a pre-typed convenience wrapper.
//!
//! # Example
//!
//! ```rust
//! use tairitsu_hooks::use_element_ref;
//!
//! let div_ref = use_element_ref::<u64>();
//!
//! assert!(div_ref.get().is_none());
//! ```

use std::cell::RefCell;
use std::rc::Rc;

/// A reference to a DOM element that will be populated during rendering.
///
/// `ElementRef` holds an optional platform element handle. Initially `None`,
/// it gets populated when the associated VNode is mounted to the DOM.
///
/// # Type Parameters
///
/// * `E` - The element handle type (e.g., `WitElement` for WIT-backed platform)
///
/// # Example
///
/// ```rust
/// use tairitsu_hooks::ElementRef;
///
/// // Create an empty ElementRef for u64 handles
/// let ref_handle: ElementRef<u64> = ElementRef::new();
///
/// // Initially, there's no element
/// assert!(ref_handle.get().is_none());
///
/// // After the element is mounted (automatically done by the framework):
/// // if let Some(element) = ref_handle.get() {
/// //     // Now you can use the element handle
/// // }
/// ```
#[derive(Clone)]
pub struct ElementRef<E>
where
    E: Clone + 'static,
{
    /// The inner handle storage, shared via Rc<RefCell>
    /// to allow cloning the ref while maintaining the same underlying reference.
    handle: Rc<RefCell<Option<E>>>,

    /// A type-erased version of the ref that can be passed to VNode.
    /// This is populated with a boxed version of the element handle when mounted.
    type_erased_handle: Rc<RefCell<Option<Box<dyn std::any::Any>>>>,
}

impl<E> ElementRef<E>
where
    E: Clone + 'static,
{
    /// Create a new empty `ElementRef`.
    ///
    /// The reference will be `None` until the associated DOM element is mounted.
    pub fn new() -> Self {
        Self {
            handle: Rc::new(RefCell::new(None)),
            type_erased_handle: Rc::new(RefCell::new(None)),
        }
    }

    /// Get a reference to the underlying element handle, if available.
    ///
    /// Returns `Some(element)` if the DOM element has been mounted,
    /// or `None` if the element hasn't been created yet.
    ///
    /// # Example
    ///
    /// ```rust
    /// use tairitsu_hooks::use_element_ref;
    ///
    /// let div_ref = use_element_ref::<u64>();
    ///
    /// // In an event handler:
    /// if let Some(element) = div_ref.get() {
    ///     // Access the element handle (u64 for WIT platform)
    ///     let handle: u64 = element;
    /// }
    /// ```
    pub fn get(&self) -> Option<E> {
        self.handle.borrow().clone()
    }

    /// Set the element handle (used internally by the framework during mounting).
    ///
    /// This method is called automatically when the VNode tree is rendered
    /// and the DOM element is created. You typically don't need to call this
    /// manually.
    pub fn set(&self, element: E) {
        *self.handle.borrow_mut() = Some(element.clone());
        // Also update the type-erased handle
        *self.type_erased_handle.borrow_mut() = Some(Box::new(element) as Box<dyn std::any::Any>);
    }

    /// Clear the element reference (used internally during unmounting/reconciliation).
    ///
    /// This is called when the element is removed from the DOM or during
    /// reconciliation when elements are replaced.
    pub fn clear(&self) {
        *self.handle.borrow_mut() = None;
        *self.type_erased_handle.borrow_mut() = None;
    }

    /// Take the element handle out of the ref, leaving `None` in its place.
    ///
    /// This is useful when transferring ownership of the element handle.
    pub fn take(&self) -> Option<E> {
        *self.type_erased_handle.borrow_mut() = None;
        self.handle.borrow_mut().take()
    }

    /// Get the type-erased handle for use with VNode.
    ///
    /// This is used internally by the rsx! macro to attach the ref to a VElement.
    pub fn as_any_ref(&self) -> Rc<RefCell<Option<Box<dyn std::any::Any>>>> {
        Rc::clone(&self.type_erased_handle)
    }
}

impl<E> Default for ElementRef<E>
where
    E: Clone + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new [`ElementRef`] hook for referencing DOM elements.
///
/// This hook creates a reference that will be automatically populated with
/// the DOM element handle when the associated VNode is mounted.
///
/// # Usage in rsx!
///
/// Use the `ref_:` attribute to bind an `ElementRef` to an element.
///
/// # Accessing the element
///
/// After the component is mounted, you can access the element:
///
/// ```rust
/// use tairitsu_hooks::use_element_ref;
///
/// // Create an element reference for u64 handles (WIT platform)
/// let div_ref = use_element_ref::<u64>();
///
/// // After mounting, access the element
/// if let Some(handle) = div_ref.get() {
///     // Use the handle (u64 for WIT platform)
/// }
/// ```
///
/// # Platform-specific element types
///
/// The element handle type depends on the platform:
/// - **WIT platform**: `tairitsu_web::WitElement` (wraps a `u64` handle)
/// - **Other platforms**: Corresponding element handle type
///
/// # Notes
///
/// - The ref is `None` until the element is mounted to the DOM
/// - The ref is automatically cleared during reconciliation if the element is replaced
/// - Multiple refs can point to the same element (though this is uncommon)
pub fn use_element_ref<E>() -> ElementRef<E>
where
    E: Clone + 'static,
{
    ElementRef::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_element_ref_new() {
        let ref_handle: ElementRef<u64> = ElementRef::new();
        assert!(ref_handle.get().is_none());
    }

    #[test]
    fn test_element_ref_default() {
        let ref_handle: ElementRef<String> = ElementRef::default();
        assert!(ref_handle.get().is_none());
    }

    #[test]
    fn test_element_ref_set_get() {
        let ref_handle: ElementRef<u64> = ElementRef::new();
        assert!(ref_handle.get().is_none());

        ref_handle.set(42);
        assert_eq!(ref_handle.get(), Some(42));
    }

    #[test]
    fn test_element_ref_clone() {
        let ref1: ElementRef<u64> = ElementRef::new();
        ref1.set(100);

        let ref2 = ref1.clone();
        assert_eq!(ref2.get(), Some(100));

        // Modifying ref2 affects ref1 (they share the same Rc<RefCell>)
        ref2.set(200);
        assert_eq!(ref1.get(), Some(200));
        assert_eq!(ref2.get(), Some(200));
    }

    #[test]
    fn test_element_ref_clear() {
        let ref_handle: ElementRef<u64> = ElementRef::new();
        ref_handle.set(42);
        assert_eq!(ref_handle.get(), Some(42));

        ref_handle.clear();
        assert!(ref_handle.get().is_none());
    }

    #[test]
    fn test_element_ref_take() {
        let ref_handle: ElementRef<u64> = ElementRef::new();
        ref_handle.set(42);
        assert_eq!(ref_handle.get(), Some(42));

        let taken = ref_handle.take();
        assert_eq!(taken, Some(42));
        assert!(ref_handle.get().is_none());
    }

    #[test]
    fn test_use_element_ref() {
        let ref_handle: ElementRef<String> = use_element_ref();
        assert!(ref_handle.get().is_none());

        ref_handle.set("test".to_string());
        assert_eq!(ref_handle.get(), Some("test".to_string()));
    }
}
