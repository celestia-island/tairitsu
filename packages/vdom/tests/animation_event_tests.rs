//! AnimationEvent tests
//!
//! Tests for AnimationEvent structure and functionality.

use tairitsu_vdom::events::AnimationEvent;

#[test]
fn test_animation_event_new_has_default_values() {
    let event = AnimationEvent::new();

    assert_eq!(event.target, None);
    assert_eq!(event.animation_name, "");
    assert_eq!(event.pseudo_element, "");
    assert_eq!(event.elapsed_time, 0.0);
    assert_eq!(event.iteration, 0.0);
}

#[test]
fn test_animation_event_builder_target() {
    let event = AnimationEvent::new().target(12345);
    assert_eq!(event.target, Some(12345));
}

#[test]
fn test_animation_event_builder_animation_name() {
    let event = AnimationEvent::new().animation_name("fadeIn");
    assert_eq!(event.animation_name, "fadeIn");
}

#[test]
fn test_animation_event_builder_pseudo_element() {
    let event = AnimationEvent::new().pseudo_element("::before");
    assert_eq!(event.pseudo_element, "::before");
}

#[test]
fn test_animation_event_builder_elapsed_time() {
    let event = AnimationEvent::new().elapsed_time(0.5);
    assert_eq!(event.elapsed_time, 0.5);
}

#[test]
fn test_animation_event_builder_iteration() {
    let event = AnimationEvent::new().iteration(2.0);
    assert_eq!(event.iteration, 2.0);
}

#[test]
fn test_animation_event_clone() {
    let event = AnimationEvent::new()
        .target(123)
        .animation_name("slideIn")
        .elapsed_time(1.0)
        .iteration(1.0);

    let cloned = event.clone();

    assert_eq!(cloned.target, event.target);
    assert_eq!(cloned.animation_name, event.animation_name);
    assert_eq!(cloned.elapsed_time, event.elapsed_time);
    assert_eq!(cloned.iteration, event.iteration);
}

#[test]
fn test_animation_event_default() {
    let event = AnimationEvent::default();

    assert_eq!(event.target, None);
    assert_eq!(event.animation_name, "");
    assert_eq!(event.elapsed_time, 0.0);
    assert_eq!(event.iteration, 0.0);
}

#[test]
fn test_animation_event_builder_chain() {
    let event = AnimationEvent::new()
        .target(123)
        .animation_name("bounce")
        .pseudo_element("::after")
        .elapsed_time(1.5)
        .iteration(3.0);

    assert_eq!(event.target, Some(123));
    assert_eq!(event.animation_name, "bounce");
    assert_eq!(event.pseudo_element, "::after");
    assert_eq!(event.elapsed_time, 1.5);
    assert_eq!(event.iteration, 3.0);
}

#[test]
fn test_animation_event_iteration_fractional() {
    let event = AnimationEvent::new().iteration(2.5);
    assert_eq!(event.iteration, 2.5);
}

#[test]
fn test_animation_event_multiple_animations() {
    let event = AnimationEvent::new()
        .animation_name("rotate")
        .elapsed_time(0.75)
        .iteration(1.5);

    assert_eq!(event.animation_name, "rotate");
    assert_eq!(event.elapsed_time, 0.75);
    assert_eq!(event.iteration, 1.5);
}
