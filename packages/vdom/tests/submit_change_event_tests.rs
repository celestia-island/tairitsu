use tairitsu_vdom::{ChangeEvent, EventData, EventWitHandle, SubmitEvent};

#[test]
fn test_submit_event_default() {
    let event = SubmitEvent::default();
    assert!(event.form_data.is_empty());
    assert!(event.target.is_none());
}

#[test]
fn test_submit_event_new() {
    let event = SubmitEvent::new();
    assert!(event.form_data.is_empty());
}

#[test]
fn test_submit_event_prevent_default_no_panic() {
    let event = SubmitEvent::new();
    event.prevent_default();
}

#[test]
fn test_submit_event_stop_propagation_no_panic() {
    let event = SubmitEvent::new();
    event.stop_propagation();
}

#[test]
fn test_submit_event_as_event_data() {
    let event = SubmitEvent::new();
    let _data: &dyn EventData = &event;
}

#[test]
fn test_submit_event_with_event_handle() {
    let handle = EventWitHandle::from_wit(42);
    let event = SubmitEvent::new().with_event_handle(handle);
    event.prevent_default();
    event.stop_propagation();
}

#[test]
fn test_submit_event_clone() {
    let mut event = SubmitEvent::new();
    event.target = Some(123);
    event
        .form_data
        .push(("key".to_string(), "value".to_string()));
    let cloned = event.clone();
    assert_eq!(cloned.target, Some(123));
    assert_eq!(cloned.form_data.len(), 1);
    assert_eq!(cloned.form_data[0].0, "key");
}

#[test]
fn test_change_event_default() {
    let event = ChangeEvent::default();
    assert_eq!(event.value, "");
    assert!(event.target.is_none());
}

#[test]
fn test_change_event_new() {
    let event = ChangeEvent::new();
    assert_eq!(event.value, "");
}

#[test]
fn test_change_event_prevent_default_no_panic() {
    let event = ChangeEvent::new();
    event.prevent_default();
}

#[test]
fn test_change_event_stop_propagation_no_panic() {
    let event = ChangeEvent::new();
    event.stop_propagation();
}

#[test]
fn test_change_event_as_event_data() {
    let event = ChangeEvent::new();
    let _data: &dyn EventData = &event;
}

#[test]
fn test_change_event_with_event_handle() {
    let handle = EventWitHandle::from_wit(99);
    let event = ChangeEvent::new().with_event_handle(handle);
    event.prevent_default();
    event.stop_propagation();
}
