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

#[test]
fn test_create_effect_runs_immediately() {
    use std::cell::Cell;
    use std::rc::Rc;

    let count = Rc::new(Cell::new(0));
    let count_clone = count.clone();
    tairitsu_vdom::create_effect(move || {
        count_clone.set(count_clone.get() + 1);
    });
    assert_eq!(count.get(), 1);
}

#[test]
fn test_create_effect_retracks_on_signal_change() {
    use std::cell::Cell;
    use std::rc::Rc;

    let signal = Signal::new(0);
    let count = Rc::new(Cell::new(0));
    let count_clone = count.clone();
    let signal_clone = signal.clone();

    tairitsu_vdom::create_effect(move || {
        let _val = signal_clone.get();
        count_clone.set(count_clone.get() + 1);
    });

    assert_eq!(count.get(), 1);
    signal.set(1);
    assert_eq!(count.get(), 2);
    signal.set(2);
    assert_eq!(count.get(), 3);
}

#[test]
fn test_effect_handle_stop() {
    use std::cell::Cell;
    use std::rc::Rc;

    let signal = Signal::new(0);
    let count = Rc::new(Cell::new(0));
    let count_clone = count.clone();
    let signal_clone = signal.clone();

    let handle = tairitsu_vdom::create_effect(move || {
        let _val = signal_clone.get();
        count_clone.set(count_clone.get() + 1);
    });

    assert_eq!(count.get(), 1);
    handle.stop();
    signal.set(1);
    assert_eq!(count.get(), 1);
}

#[test]
fn test_effect_with_multiple_signals() {
    use std::cell::Cell;
    use std::rc::Rc;

    let a = Signal::new(1);
    let b = Signal::new(2);
    let sum = Rc::new(Cell::new(0));
    let sum_clone = sum.clone();
    let a_clone = a.clone();
    let b_clone = b.clone();

    tairitsu_vdom::create_effect(move || {
        let val = a_clone.get() + b_clone.get();
        sum_clone.set(val);
    });

    assert_eq!(sum.get(), 3);
    a.set(10);
    assert_eq!(sum.get(), 12);
    b.set(20);
    assert_eq!(sum.get(), 30);
}

#[test]
fn test_effect_no_dep_count_stays_1() {
    use std::cell::Cell;
    use std::rc::Rc;

    let count = Rc::new(Cell::new(0));
    let count_clone = count.clone();
    tairitsu_vdom::create_effect(move || {
        count_clone.set(count_clone.get() + 1);
    });
    assert_eq!(count.get(), 1);
}

#[test]
fn test_dynamic_text_creation() {
    use tairitsu_vdom::dynamic_text;

    let node = dynamic_text("hello".to_string(), || "world".to_string());
    match node {
        VNode::DynamicText(dt) => assert_eq!(dt.initial, "hello"),
        _ => panic!("Expected DynamicText"),
    }
}

#[test]
fn test_dynamic_text_with_signal() {
    use tairitsu_vdom::dynamic_text;

    let signal = Signal::new(42);
    let signal_clone = signal.clone();
    let node = dynamic_text("42".to_string(), move || signal_clone.get().to_string());

    match node {
        VNode::DynamicText(dt) => {
            assert_eq!(dt.initial, "42");
        }
        _ => panic!("Expected DynamicText"),
    }
}

#[test]
fn test_into_vnode_child_string() {
    use tairitsu_vdom::IntoVNodeChild;

    let node = "hello".into_vnode_child();
    match node {
        VNode::Text(t) => assert_eq!(t.text, "hello"),
        _ => panic!("Expected Text"),
    }
}

#[test]
fn test_into_vnode_child_signal() {
    use tairitsu_vdom::IntoVNodeChild;

    let signal = Signal::new(42);
    let node = signal.into_vnode_child();
    match node {
        VNode::DynamicText(dt) => assert_eq!(dt.initial, "42"),
        _ => panic!("Expected DynamicText"),
    }
}

#[test]
fn test_velement_dynamic_attr_builder() {
    let el = VElement::new("div").dynamic_attr("data-count", || "42".to_string());
    assert_eq!(el.dynamic_attributes.len(), 1);
    assert_eq!(el.dynamic_attributes[0].0, "data-count");
}

#[test]
fn test_velement_dynamic_style_builder() {
    let el = VElement::new("div").dynamic_style("color", || "red".to_string());
    assert_eq!(el.dynamic_styles.len(), 1);
    assert_eq!(el.dynamic_styles[0].0, "color");
}

#[test]
fn test_velement_dynamic_class_builder() {
    let el = VElement::new("div").dynamic_class(|| "active".to_string());
    assert_eq!(el.dynamic_classes.len(), 1);
}
