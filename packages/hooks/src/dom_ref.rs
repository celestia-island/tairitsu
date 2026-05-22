//! Typed DOM element references for imperative manipulation.
//!
//! Unlike [`use_element_ref`](super::element_ref::use_element_ref) which is
//! generic over any type, [`use_dom_ref`] returns handles that are already
//! resolved to [`DomHandle`](tairitsu_vdom::DomHandle), eliminating the
//! unsafe downcast dance entirely.
//!
//! # Example
//!
//! ```ignore
//! let my_ref = use_dom_ref();
//! let btn = VElement::new("button")
//!     .ref_(my_ref.as_any_ref().clone())
//!     .on_event("click", move |_e| {
//!         if let Some(h) = my_ref.get() {
//!             set_style(h, "color", "red");
//!         }
//!     });
//! ```

use std::{cell::RefCell, rc::Rc};

use tairitsu_vdom::{resolve_element_ref, AnyElementRef, DomHandle};

/// A DOM element reference that resolves to a [`DomHandle`] after mounting.
///
/// This is the recommended way to imperatively manipulate elements
/// rendered by the VDOM.
#[derive(Clone)]
pub struct DomRef {
    inner: Rc<RefCell<Option<DomHandle>>>,
    any_ref: AnyElementRef,
}

impl DomRef {
    /// Get the current [`DomHandle`], if the element has been mounted.
    pub fn get(&self) -> Option<DomHandle> {
        let cached = *self.inner.borrow();
        if cached.is_some() {
            return cached;
        }
        let resolved = resolve_element_ref(&self.any_ref);
        if resolved.is_some() {
            *self.inner.borrow_mut() = resolved;
        }
        resolved
    }

    /// Get the type-erased ref for passing to [`VElement::ref_()`](tairitsu_vdom::VElement::ref_).
    pub fn as_any_ref(&self) -> &AnyElementRef {
        &self.any_ref
    }

    /// Clear the cached handle (e.g. after unmount).
    pub fn clear(&self) {
        *self.inner.borrow_mut() = None;
    }
}

/// Create a new empty [`DomRef`].
pub fn use_dom_ref() -> DomRef {
    DomRef {
        inner: Rc::new(RefCell::new(None)),
        any_ref: Rc::new(RefCell::new(None)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dom_ref_new() {
        let r = use_dom_ref();
        assert!(r.get().is_none());
    }

    #[test]
    fn test_dom_ref_clone_shares_state() {
        let r1 = use_dom_ref();
        let r2 = r1.clone();
        assert!(r1.get().is_none());
        assert!(r2.get().is_none());
    }
}
