//! MouseEvent field tests
//!
//! Tests for MouseEvent structure fields including new coordinate fields.

use tairitsu_vdom::events::{EventWitHandle, MouseEvent};

#[test]
fn test_mouse_event_new_has_default_values() {
    let event = MouseEvent::new();

    // Check all coordinate fields are zero
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

    // Check other fields
    assert_eq!(event.target, None);
    assert_eq!(event.button, 0);
    assert_eq!(event.buttons, 0);
    assert!(!event.ctrl_key);
    assert!(!event.shift_key);
    assert!(!event.alt_key);
    assert!(!event.meta_key);
}

#[test]
fn test_mouse_event_builder_target() {
    let event = MouseEvent::new().target(12345);

    assert_eq!(event.target, Some(12345));
}

#[test]
fn test_mouse_event_builder_client_x() {
    let event = MouseEvent::new().client_x(100);

    assert_eq!(event.client_x, 100);
}

#[test]
fn test_mouse_event_builder_client_y() {
    let event = MouseEvent::new().client_y(200);

    assert_eq!(event.client_y, 200);
}

#[test]
fn test_mouse_event_builder_event_handle() {
    let handle = EventWitHandle::from_wit(999);
    let event = MouseEvent::new().event_handle(handle);

    // The handle should be set internally
}

#[test]
fn test_mouse_event_all_coordinate_fields_exist() {
    // This test verifies that all new coordinate fields exist
    // Using the builder pattern which is the proper way to construct MouseEvent

    let event = MouseEvent::new()
        .target(123)
        .client_x(10)
        .client_y(20)
        .screen_x(30)
        .screen_y(40)
        .offset_x(50)
        .offset_y(60)
        .page_x(70)
        .page_y(80)
        .movement_x(90)
        .movement_y(100);

    assert_eq!(event.target, Some(123));
    assert_eq!(event.client_x, 10);
    assert_eq!(event.client_y, 20);
    assert_eq!(event.screen_x, 30);
    assert_eq!(event.screen_y, 40);
    assert_eq!(event.offset_x, 50);
    assert_eq!(event.offset_y, 60);
    assert_eq!(event.page_x, 70);
    assert_eq!(event.page_y, 80);
    assert_eq!(event.movement_x, 90);
    assert_eq!(event.movement_y, 100);
}

#[test]
fn test_mouse_event_clone() {
    let event = MouseEvent::new()
        .target(123)
        .client_x(100)
        .client_y(200);

    let cloned = event.clone();

    assert_eq!(cloned.target, event.target);
    assert_eq!(cloned.client_x, event.client_x);
    assert_eq!(cloned.client_y, event.client_y);
}

#[test]
fn test_mouse_event_default() {
    let event = MouseEvent::default();

    assert_eq!(event.client_x, 0);
    assert_eq!(event.client_y, 0);
    assert_eq!(event.target, None);
}
