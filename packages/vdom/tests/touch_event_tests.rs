//! TouchEvent tests
//!
//! Tests for TouchEvent structure and functionality.

use tairitsu_vdom::events::{TouchEvent, TouchPoint};

#[test]
fn test_touch_point_default() {
    let point = TouchPoint::default();

    assert_eq!(point.identifier, 0);
    assert_eq!(point.client_x, 0);
    assert_eq!(point.client_y, 0);
    assert_eq!(point.screen_x, 0);
    assert_eq!(point.screen_y, 0);
    assert_eq!(point.page_x, 0);
    assert_eq!(point.page_y, 0);
    assert_eq!(point.target, None);
    assert_eq!(point.force, 0.0);
    assert_eq!(point.radius_x, 0.0);
    assert_eq!(point.radius_y, 0.0);
    assert_eq!(point.rotation_angle, 0.0);
}

#[test]
fn test_touch_event_new_has_default_values() {
    let event = TouchEvent::new();

    assert_eq!(event.target, None);
    assert!(event.touches.is_empty());
    assert!(event.changed_touches.is_empty());
    assert!(event.target_touches.is_empty());
    assert_eq!(event.timestamp, 0.0);
}

#[test]
fn test_touch_event_builder_target() {
    let event = TouchEvent::new().target(12345);
    assert_eq!(event.target, Some(12345));
}

#[test]
fn test_touch_event_builder_touches() {
    let touches = vec![
        TouchPoint {
            identifier: 1,
            client_x: 100,
            client_y: 200,
            ..Default::default()
        },
        TouchPoint {
            identifier: 2,
            client_x: 150,
            client_y: 250,
            ..Default::default()
        },
    ];

    let event = TouchEvent::new().touches(touches.clone());

    assert_eq!(event.touches.len(), 2);
    assert_eq!(event.touches[0].identifier, 1);
    assert_eq!(event.touches[1].identifier, 2);
}

#[test]
fn test_touch_event_builder_changed_touches() {
    let changed = vec![TouchPoint {
        identifier: 1,
        client_x: 100,
        client_y: 200,
        ..Default::default()
    }];

    let event = TouchEvent::new().changed_touches(changed);

    assert_eq!(event.changed_touches.len(), 1);
    assert_eq!(event.changed_touches[0].identifier, 1);
}

#[test]
fn test_touch_event_builder_target_touches() {
    let target_touches = vec![TouchPoint {
        identifier: 1,
        client_x: 100,
        client_y: 200,
        ..Default::default()
    }];

    let event = TouchEvent::new().target_touches(target_touches);

    assert_eq!(event.target_touches.len(), 1);
    assert_eq!(event.target_touches[0].identifier, 1);
}

#[test]
fn test_touch_event_builder_timestamp() {
    let event = TouchEvent::new().timestamp(12345.67);
    assert_eq!(event.timestamp, 12345.67);
}

#[test]
fn test_touch_event_clone() {
    let touches = vec![TouchPoint {
        identifier: 1,
        client_x: 100,
        client_y: 200,
        ..Default::default()
    }];

    let event = TouchEvent::new()
        .target(123)
        .touches(touches)
        .timestamp(1000.0);

    let cloned = event.clone();

    assert_eq!(cloned.target, event.target);
    assert_eq!(cloned.touches.len(), event.touches.len());
    assert_eq!(cloned.timestamp, event.timestamp);
}

#[test]
fn test_touch_event_default() {
    let event = TouchEvent::default();

    assert_eq!(event.target, None);
    assert!(event.touches.is_empty());
    assert_eq!(event.timestamp, 0.0);
}

#[test]
fn test_touch_point_full() {
    let point = TouchPoint {
        identifier: 42,
        client_x: 100,
        client_y: 200,
        screen_x: 300,
        screen_y: 400,
        page_x: 500,
        page_y: 600,
        target: Some(789),
        force: 0.5,
        radius_x: 10.0,
        radius_y: 15.0,
        rotation_angle: 45.0,
    };

    assert_eq!(point.identifier, 42);
    assert_eq!(point.client_x, 100);
    assert_eq!(point.client_y, 200);
    assert_eq!(point.screen_x, 300);
    assert_eq!(point.screen_y, 400);
    assert_eq!(point.page_x, 500);
    assert_eq!(point.page_y, 600);
    assert_eq!(point.target, Some(789));
    assert_eq!(point.force, 0.5);
    assert_eq!(point.radius_x, 10.0);
    assert_eq!(point.radius_y, 15.0);
    assert_eq!(point.rotation_angle, 45.0);
}
