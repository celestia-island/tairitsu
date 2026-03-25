//! Integration test for `use_element_ref` hook.
//!
//! This test verifies that element refs work correctly.

use std::{cell::RefCell, rc::Rc};

use tairitsu_hooks::use_element_ref;

#[test]
fn test_element_ref_basic() {
    let ref_handle: Rc<RefCell<Option<Box<dyn std::any::Any>>>> = Rc::new(RefCell::new(None));

    // Initially, the ref is None
    assert!(ref_handle.borrow().is_none());

    // Simulate what happens during mounting - set the element
    *ref_handle.borrow_mut() = Some(Box::new(42u64) as Box<dyn std::any::Any>);

    // Now the ref has a value
    let ref_value = ref_handle.borrow();
    assert!(ref_value.is_some());
    let value = ref_value.as_ref().unwrap();
    if let Some(num) = value.downcast_ref::<u64>() {
        assert_eq!(*num, 42);
    } else {
        panic!("Failed to downcast to u64");
    }
}

#[test]
fn test_element_ref_clone_shares_state() {
    let ref1 = use_element_ref::<String>();
    let ref2 = ref1.clone();

    // Both refs are initially None
    assert!(ref1.get().is_none());
    assert!(ref2.get().is_none());

    // Set value on ref1
    ref1.set("test".to_string());

    // Both refs should see the value (they share the same Rc<RefCell>)
    assert_eq!(ref1.get(), Some("test".to_string()));
    assert_eq!(ref2.get(), Some("test".to_string()));

    // Update via ref2
    ref2.set("updated".to_string());

    // Both should see the updated value
    assert_eq!(ref1.get(), Some("updated".to_string()));
    assert_eq!(ref2.get(), Some("updated".to_string()));
}

#[test]
fn test_element_ref_clear() {
    let ref_handle = use_element_ref::<u64>();

    ref_handle.set(42);
    assert_eq!(ref_handle.get(), Some(42));

    ref_handle.clear();
    assert!(ref_handle.get().is_none());
}

#[test]
fn test_element_ref_take() {
    let ref_handle = use_element_ref::<String>();

    ref_handle.set("test".to_string());
    assert_eq!(ref_handle.get(), Some("test".to_string()));

    let taken = ref_handle.take();
    assert_eq!(taken, Some("test".to_string()));

    // After taking, the ref should be None
    assert!(ref_handle.get().is_none());
}

#[test]
fn test_element_ref_default() {
    let ref_handle: tairitsu_hooks::ElementRef<u64> = Default::default();
    assert!(ref_handle.get().is_none());
}

#[test]
fn test_element_ref_as_any_ref() {
    let ref_handle = use_element_ref::<u64>();

    // Get the type-erased ref
    let any_ref = ref_handle.as_any_ref();

    // Initially None
    assert!(any_ref.borrow().is_none());

    // Set a value
    ref_handle.set(123);

    // The type-erased ref should also have the value
    let borrowed = any_ref.borrow();
    assert!(borrowed.is_some());
    let value = borrowed.as_ref().unwrap();
    if let Some(num) = value.downcast_ref::<u64>() {
        assert_eq!(*num, 123);
    } else {
        panic!("Failed to downcast to u64");
    }
}
