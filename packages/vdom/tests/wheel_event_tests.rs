//! WheelEvent tests
//!
//! Tests for WheelEvent structure and functionality.

use tairitsu_vdom::events::WheelEvent;

#[test]
fn test_wheel_event_new_has_default_values() {
    let event = WheelEvent::new();

    assert_eq!(event.delta_x, 0.0);
    assert_eq!(event.delta_y, 0.0);
    assert_eq!(event.delta_z, 0.0);
    assert_eq!(event.delta_mode, 0);
    assert_eq!(event.client_x, 0);
    assert_eq!(event.client_y, 0);
    assert_eq!(event.screen_x, 0);
    assert_eq!(event.screen_y, 0);
    assert!(!event.ctrl_key);
    assert!(!event.shift_key);
    assert!(!event.alt_key);
    assert!(!event.meta_key);
    assert_eq!(event.target, None);
}

#[test]
fn test_wheel_event_builder_target() {
    let event = WheelEvent::new().target(12345);
    assert_eq!(event.target, Some(12345));
}

#[test]
fn test_wheel_event_builder_deltas() {
    let event = WheelEvent::new()
        .delta_x(10.5)
        .delta_y(-20.3)
        .delta_z(5.0);

    assert_eq!(event.delta_x, 10.5);
    assert_eq!(event.delta_y, -20.3);
    assert_eq!(event.delta_z, 5.0);
}

#[test]
fn test_wheel_event_builder_delta_mode() {
    let event = WheelEvent::new().delta_mode(1);
    assert_eq!(event.delta_mode, 1);
}

#[test]
fn test_wheel_event_builder_client_coords() {
    let event = WheelEvent::new()
        .client_x(100)
        .client_y(200);

    assert_eq!(event.client_x, 100);
    assert_eq!(event.client_y, 200);
}

#[test]
fn test_wheel_event_builder_screen_coords() {
    let event = WheelEvent::new()
        .screen_x(300)
        .screen_y(400);

    assert_eq!(event.screen_x, 300);
    assert_eq!(event.screen_y, 400);
}

#[test]
fn test_wheel_event_builder_modifiers() {
    let event = WheelEvent::new()
        .ctrl_key(true)
        .shift_key(true)
        .alt_key(true)
        .meta_key(true);

    assert!(event.ctrl_key);
    assert!(event.shift_key);
    assert!(event.alt_key);
    assert!(event.meta_key);
}

#[test]
fn test_wheel_event_clone() {
    let event = WheelEvent::new()
        .target(123)
        .delta_x(10.5)
        .delta_y(-20.3);

    let cloned = event.clone();

    assert_eq!(cloned.target, event.target);
    assert_eq!(cloned.delta_x, event.delta_x);
    assert_eq!(cloned.delta_y, event.delta_y);
}

#[test]
fn test_wheel_event_default() {
    let event = WheelEvent::default();

    assert_eq!(event.delta_x, 0.0);
    assert_eq!(event.delta_y, 0.0);
    assert_eq!(event.target, None);
}

#[test]
fn test_wheel_event_builder_chain() {
    let event = WheelEvent::new()
        .target(123)
        .delta_x(10.5)
        .delta_y(-20.3)
        .delta_mode(1)
        .client_x(100)
        .client_y(200)
        .screen_x(300)
        .screen_y(400)
        .ctrl_key(true)
        .shift_key(true);

    assert_eq!(event.target, Some(123));
    assert_eq!(event.delta_x, 10.5);
    assert_eq!(event.delta_y, -20.3);
    assert_eq!(event.delta_mode, 1);
    assert_eq!(event.client_x, 100);
    assert_eq!(event.client_y, 200);
    assert_eq!(event.screen_x, 300);
    assert_eq!(event.screen_y, 400);
    assert!(event.ctrl_key);
    assert!(event.shift_key);
}
