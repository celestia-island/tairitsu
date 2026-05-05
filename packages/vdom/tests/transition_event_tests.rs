//! TransitionEvent tests
//!
//! Tests for TransitionEvent structure and functionality.

use tairitsu_vdom::events::TransitionEvent;

#[test]
fn test_transition_event_new_has_default_values() {
    let event = TransitionEvent::new();

    assert_eq!(event.target, None);
    assert_eq!(event.property_name, "");
    assert_eq!(event.elapsed_time, 0.0);
    assert_eq!(event.pseudo_element, "");
}

#[test]
fn test_transition_event_builder_target() {
    let event = TransitionEvent::new().target(12345);
    assert_eq!(event.target, Some(12345));
}

#[test]
fn test_transition_event_builder_property_name() {
    let event = TransitionEvent::new().property_name("opacity");
    assert_eq!(event.property_name, "opacity");
}

#[test]
fn test_transition_event_builder_elapsed_time() {
    let event = TransitionEvent::new().elapsed_time(0.3);
    assert_eq!(event.elapsed_time, 0.3);
}

#[test]
fn test_transition_event_builder_pseudo_element() {
    let event = TransitionEvent::new().pseudo_element("::before");
    assert_eq!(event.pseudo_element, "::before");
}

#[test]
fn test_transition_event_clone() {
    let event = TransitionEvent::new()
        .target(123)
        .property_name("opacity")
        .elapsed_time(0.5);

    let cloned = event.clone();

    assert_eq!(cloned.target, event.target);
    assert_eq!(cloned.property_name, event.property_name);
    assert_eq!(cloned.elapsed_time, event.elapsed_time);
}

#[test]
fn test_transition_event_default() {
    let event = TransitionEvent::default();

    assert_eq!(event.target, None);
    assert_eq!(event.property_name, "");
    assert_eq!(event.elapsed_time, 0.0);
}

#[test]
fn test_transition_event_builder_chain() {
    let event = TransitionEvent::new()
        .target(123)
        .property_name("transform")
        .elapsed_time(0.3)
        .pseudo_element("::after");

    assert_eq!(event.target, Some(123));
    assert_eq!(event.property_name, "transform");
    assert_eq!(event.elapsed_time, 0.3);
    assert_eq!(event.pseudo_element, "::after");
}

#[test]
fn test_transition_event_multiple_properties() {
    let event = TransitionEvent::new()
        .property_name("background-color")
        .elapsed_time(1.5)
        .pseudo_element("");

    assert_eq!(event.property_name, "background-color");
    assert_eq!(event.elapsed_time, 1.5);
    assert_eq!(event.pseudo_element, "");
}
