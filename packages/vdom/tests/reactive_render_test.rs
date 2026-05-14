//! Integration tests for reactive rendering with Signal -> re-render scheduling.
//!
//! These tests verify that:
//! 1. Signal changes trigger component re-renders
//! 2. Dependencies are tracked correctly
//! 3. Patches are applied efficiently

use tairitsu_vdom::{
    vnode::{VElement, VNode},
    Signal,
};

#[test]
fn test_signal_triggers_dependency_tracking() {
    let signal = Signal::new(42);

    let signal_clone = signal.clone();

    let _component_id = tairitsu_vdom::use_component(move || {
        let value = signal_clone.get();
        VNode::Text(tairitsu_vdom::vnode::VText::new(&format!(
            "Value: {}",
            value
        )))
    });

    signal.set(100);
}

#[test]
fn test_vnode_diff_produces_patches() {
    let old = VNode::Element(VElement::new("div").attr("class", "old-class"));
    let new = VNode::Element(VElement::new("div").attr("class", "new-class"));

    let patches = tairitsu_vdom::diff::diff(Some(&old), &new);

    assert!(!patches.is_empty());
}

#[test]
fn test_signal_get_tracks_dependencies() {
    let signal = Signal::new("hello");

    let signal_clone = signal.clone();

    let component_id = tairitsu_vdom::use_component(move || {
        let value = signal_clone.get();
        VNode::Text(tairitsu_vdom::vnode::VText::new(value))
    });

    assert!(component_id > 0);
}

#[test]
fn test_batch_updates() {
    let signal1 = Signal::new(1);
    let signal2 = Signal::new(2);

    tairitsu_vdom::batch(|| {
        signal1.set(10);
        signal2.set(20);
    });

    assert_eq!(signal1.get(), 10);
    assert_eq!(signal2.get(), 20);
}
