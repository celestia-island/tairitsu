use tairitsu_vdom::{ChangeEvent, EventData, SubmitEvent};

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
