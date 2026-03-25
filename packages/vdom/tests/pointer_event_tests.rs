//! PointerEvent tests
//!
//! Tests for PointerEvent structure and functionality.

use tairitsu_vdom::events::{PointerEvent, PointerType};

#[test]
fn test_pointer_event_new_has_default_values() {
    let event = PointerEvent::new();

    assert_eq!(event.target, None);
    assert_eq!(event.pointer_id, 0);
    assert_eq!(event.pointer_type, PointerType::Mouse);
    assert!(!event.is_primary);
    assert_eq!(event.client_x, 0);
    assert_eq!(event.client_y, 0);
    assert_eq!(event.screen_x, 0);
    assert_eq!(event.screen_y, 0);
    assert_eq!(event.offset_x, 0);
    assert_eq!(event.offset_y, 0);
    assert_eq!(event.page_x, 0);
    assert_eq!(event.page_y, 0);
    assert_eq!(event.movement_x, 0);
    assert_eq!(event.movement_y, 0);
    assert_eq!(event.width, 0.0);
    assert_eq!(event.height, 0.0);
    assert_eq!(event.pressure, 0.0);
    assert_eq!(event.tangential_pressure, 0.0);
    assert_eq!(event.tilt_x, 0);
    assert_eq!(event.tilt_y, 0);
    assert_eq!(event.twist, 0);
    assert_eq!(event.button, 0);
    assert_eq!(event.buttons, 0);
    assert!(!event.ctrl_key);
    assert!(!event.shift_key);
    assert!(!event.alt_key);
    assert!(!event.meta_key);
}

#[test]
fn test_pointer_event_builder_target() {
    let event = PointerEvent::new().target(12345);
    assert_eq!(event.target, Some(12345));
}

#[test]
fn test_pointer_event_builder_pointer_id() {
    let event = PointerEvent::new().pointer_id(42);
    assert_eq!(event.pointer_id, 42);
}

#[test]
fn test_pointer_event_builder_pointer_type() {
    let event = PointerEvent::new().pointer_type(PointerType::Touch);
    assert_eq!(event.pointer_type, PointerType::Touch);
}

#[test]
fn test_pointer_event_builder_is_primary() {
    let event = PointerEvent::new().is_primary(true);
    assert!(event.is_primary);
}

#[test]
fn test_pointer_event_builder_coords() {
    let event = PointerEvent::new()
        .client_x(100)
        .client_y(200)
        .screen_x(300)
        .screen_y(400)
        .offset_x(50)
        .offset_y(60)
        .page_x(70)
        .page_y(80);

    assert_eq!(event.client_x, 100);
    assert_eq!(event.client_y, 200);
    assert_eq!(event.screen_x, 300);
    assert_eq!(event.screen_y, 400);
    assert_eq!(event.offset_x, 50);
    assert_eq!(event.offset_y, 60);
    assert_eq!(event.page_x, 70);
    assert_eq!(event.page_y, 80);
}

#[test]
fn test_pointer_event_builder_movement() {
    let event = PointerEvent::new()
        .movement_x(10)
        .movement_y(20);

    assert_eq!(event.movement_x, 10);
    assert_eq!(event.movement_y, 20);
}

#[test]
fn test_pointer_event_builder_dimensions() {
    let event = PointerEvent::new()
        .width(15.5)
        .height(20.3);

    assert_eq!(event.width, 15.5);
    assert_eq!(event.height, 20.3);
}

#[test]
fn test_pointer_event_builder_pressure() {
    let event = PointerEvent::new()
        .pressure(0.5)
        .tangential_pressure(0.3);

    assert_eq!(event.pressure, 0.5);
    assert_eq!(event.tangential_pressure, 0.3);
}

#[test]
fn test_pointer_event_builder_tilt() {
    let event = PointerEvent::new()
        .tilt_x(30)
        .tilt_y(-15);

    assert_eq!(event.tilt_x, 30);
    assert_eq!(event.tilt_y, -15);
}

#[test]
fn test_pointer_event_builder_twist() {
    let event = PointerEvent::new().twist(180);
    assert_eq!(event.twist, 180);
}

#[test]
fn test_pointer_event_builder_buttons() {
    let event = PointerEvent::new()
        .button(1)
        .buttons(3);

    assert_eq!(event.button, 1);
    assert_eq!(event.buttons, 3);
}

#[test]
fn test_pointer_event_builder_modifiers() {
    let event = PointerEvent::new()
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
fn test_pointer_event_clone() {
    let event = PointerEvent::new()
        .target(123)
        .pointer_id(42)
        .pointer_type(PointerType::Pen);

    let cloned = event.clone();

    assert_eq!(cloned.target, event.target);
    assert_eq!(cloned.pointer_id, event.pointer_id);
    assert_eq!(cloned.pointer_type, event.pointer_type);
}

#[test]
fn test_pointer_event_default() {
    let event = PointerEvent::default();

    assert_eq!(event.target, None);
    assert_eq!(event.pointer_id, 0);
    assert_eq!(event.pointer_type, PointerType::Mouse);
}

#[test]
fn test_pointer_type_from_str() {
    assert_eq!(PointerType::from_str("mouse"), PointerType::Mouse);
    assert_eq!(PointerType::from_str("pen"), PointerType::Pen);
    assert_eq!(PointerType::from_str("touch"), PointerType::Touch);
    assert_eq!(PointerType::from_str("unknown"), PointerType::Mouse); // fallback
}

#[test]
fn test_pointer_type_as_str() {
    assert_eq!(PointerType::Mouse.as_str(), "mouse");
    assert_eq!(PointerType::Pen.as_str(), "pen");
    assert_eq!(PointerType::Touch.as_str(), "touch");
}

#[test]
fn test_pointer_event_builder_chain() {
    let event = PointerEvent::new()
        .target(123)
        .pointer_id(42)
        .pointer_type(PointerType::Touch)
        .is_primary(true)
        .client_x(100)
        .client_y(200)
        .pressure(0.7)
        .ctrl_key(true);

    assert_eq!(event.target, Some(123));
    assert_eq!(event.pointer_id, 42);
    assert_eq!(event.pointer_type, PointerType::Touch);
    assert!(event.is_primary);
    assert_eq!(event.client_x, 100);
    assert_eq!(event.client_y, 200);
    assert_eq!(event.pressure, 0.7);
    assert!(event.ctrl_key);
}
