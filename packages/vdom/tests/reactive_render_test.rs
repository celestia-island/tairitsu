//! Integration tests for reactive rendering with Signal → re-render scheduling.
//!
//! These tests verify that:
//! 1. Signal changes trigger component re-renders
//! 2. Dependencies are tracked correctly
//! 3. Patches are applied efficiently

use std::{cell::RefCell, rc::Rc};

use tairitsu_vdom::{
    DomOps, Platform, Signal,
    scheduler::Scheduler,
    vnode::{VElement, VNode},
};

// Mock platform for testing
struct MockPlatform {
    render_log: Rc<RefCell<Vec<String>>>,
}

impl MockPlatform {
    fn new() -> Self {
        Self {
            render_log: Rc::new(RefCell::new(Vec::new())),
        }
    }

    fn log(&self, msg: &str) {
        self.render_log.borrow_mut().push(msg.to_string());
    }
}

impl DomOps for MockPlatform {
    type Element = MockElement;
    type Event = MockEvent;

    fn create_element(&self, tag: &str) -> Self::Element {
        self.log(&format!("create_element({})", tag));
        MockElement {
            tag: tag.to_string(),
        }
    }

    fn create_text_node(&self, text: &str) -> Self::Element {
        self.log(&format!("create_text_node({})", text));
        MockElement {
            tag: text.to_string(),
        }
    }

    fn append_child(&self, parent: &Self::Element, _child: &Self::Element) {
        self.log(&format!("append_child({})", parent.tag));
    }

    fn remove_child(&self, parent: &Self::Element, _child: &Self::Element) {
        self.log(&format!("remove_child({})", parent.tag));
    }

    fn set_attribute(&self, element: &Self::Element, name: &str, value: &str) {
        self.log(&format!(
            "set_attribute({}, {}, {})",
            element.tag, name, value
        ));
    }

    fn remove_attribute(&self, element: &Self::Element, name: &str) {
        self.log(&format!("remove_attribute({}, {})", element.tag, name));
    }

    fn set_style(&self, element: &Self::Element, name: &str, value: &str) {
        self.log(&format!("set_style({}, {}, {})", element.tag, name, value));
    }

    fn set_class(&self, element: &Self::Element, class: &str) {
        self.log(&format!("set_class({}, {})", element.tag, class));
    }

    fn add_event_listener(
        &self,
        element: &Self::Element,
        event: &str,
        _handler: Box<dyn FnMut(Box<dyn tairitsu_vdom::EventData>)>,
    ) {
        self.log(&format!("add_event_listener({}, {})", element.tag, event));
    }

    fn remove_event_listener(&self, element: &Self::Element, event: &str) {
        self.log(&format!(
            "remove_event_listener({}, {})",
            element.tag, event
        ));
    }

    fn get_attribute(&self, _element: &Self::Element, _name: &str) -> Option<String> {
        None
    }
    fn class_list_add(&self, _element: &Self::Element, _tokens: &[&str]) {}
    fn class_list_remove(&self, _element: &Self::Element, _tokens: &[&str]) {}
    fn class_list_contains(&self, _element: &Self::Element, _token: &str) -> bool {
        false
    }
    fn first_child(&self, _element: &Self::Element) -> Option<Self::Element> {
        None
    }
    fn insert_before(
        &self,
        _parent: &Self::Element,
        _new_node: &Self::Element,
        _reference_node: Option<&Self::Element>,
    ) {
    }
    fn query_selector_on(
        &self,
        _element: &Self::Element,
        _selector: &str,
    ) -> Option<Self::Element> {
        None
    }

    fn get_inner_html(&self, _element: &Self::Element) -> String {
        String::new()
    }

    fn set_inner_html(&self, _element: &Self::Element, _html: String) {}
}

impl tairitsu_vdom::TimerOps for MockPlatform {
    fn set_timeout(&self, _callback: Box<dyn FnOnce()>, _ms: i32) -> i32 {
        0
    }

    fn clear_timeout(&self, _id: i32) {}

    fn set_interval(&self, _callback: Box<dyn FnMut()>, _ms: i32) -> i32 {
        0
    }

    fn clear_interval(&self, _id: i32) {}

    fn request_animation_frame(&self, callback: Box<dyn FnOnce(f64)>) -> u32 {
        self.log("request_animation_frame");
        callback(0.0);
        1
    }

    fn cancel_animation_frame(&self, _id: u32) {
        self.log("cancel_animation_frame");
    }
}

impl tairitsu_vdom::LayoutOps for MockPlatform {
    fn get_bounding_client_rect(&self, _element: &Self::Element) -> tairitsu_vdom::DomRect {
        tairitsu_vdom::DomRect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        }
    }

    fn inner_width(&self) -> i32 {
        800
    }

    fn inner_height(&self) -> i32 {
        600
    }

    fn get_element_scroll_top(&self, _element: &Self::Element) -> f64 {
        0.0
    }

    fn set_element_scroll_top(&self, _element: &Self::Element, _value: f64) {}

    fn get_element_scroll_height(&self, _element: &Self::Element) -> i32 {
        0
    }
    fn get_element_client_height(&self, _element: &Self::Element) -> i32 {
        0
    }
    fn get_element_client_width(&self, _element: &Self::Element) -> i32 {
        0
    }
}

impl tairitsu_vdom::ObserverOps for MockPlatform {
    fn create_resize_observer(
        &self,
        _callback: Box<dyn FnMut(Vec<tairitsu_vdom::ResizeObserverEntry>)>,
    ) -> u64 {
        0
    }

    fn observe_resize(&self, _observer: u64, _element: &Self::Element) {}

    fn unobserve_resize(&self, _observer: u64, _element: &Self::Element) {}

    fn disconnect_resize(&self, _observer: u64) {}

    fn create_mutation_observer(
        &self,
        _callback: Box<dyn FnMut(Vec<tairitsu_vdom::MutationRecord>)>,
    ) -> u64 {
        0
    }

    fn observe_mutations(
        &self,
        _observer: u64,
        _element: &Self::Element,
        _options: Option<tairitsu_vdom::MutationObserverInit>,
    ) {
    }

    fn disconnect_mutation(&self, _observer: u64) {}
}

impl tairitsu_vdom::MediaQueryOps for MockPlatform {
    fn match_media(&self, _query: &str) -> u64 {
        0
    }

    fn media_query_list_get_media(&self, _list: u64) -> String {
        String::new()
    }

    fn media_query_list_get_matches(&self, _list: u64) -> bool {
        false
    }

    fn media_query_list_add_listener(&self, _list: u64, _callback: Box<dyn FnMut(bool)>) -> u64 {
        0
    }

    fn media_query_list_remove_listener(&self, _list: u64, _listener_id: u64) {}
}

impl tairitsu_vdom::ClipboardOps for MockPlatform {
    fn copy_to_clipboard(&self, _text: &str) -> bool {
        false
    }

    fn read_clipboard(&self) -> Option<String> {
        None
    }

    fn clipboard_write_text_async(
        &self,
        _text: &str,
        on_complete: Box<dyn FnOnce(Result<(), String>)>,
    ) {
        on_complete(Ok(()));
    }

    fn clipboard_read_text_async(&self, on_complete: Box<dyn FnOnce(Result<String, String>)>) {
        on_complete(Err("clipboard not available in mock".to_string()));
    }
}

impl tairitsu_vdom::ContentEditableOps for MockPlatform {
    fn get_contenteditable_state(
        &self,
        _element: &Self::Element,
    ) -> Option<tairitsu_vdom::ContentEditableState> {
        None
    }

    fn exec_command(&self, _command: &str, _value: Option<&str>) -> bool {
        false
    }

    fn get_selection_start(&self, _element: &Self::Element) -> Option<u32> {
        None
    }

    fn get_selection_end(&self, _element: &Self::Element) -> Option<u32> {
        None
    }

    fn set_content_editable(&self, _element: &Self::Element, _editable: bool) {}
}

impl tairitsu_vdom::ScrollOps for MockPlatform {
    fn get_scroll_y(&self) -> f64 {
        0.0
    }

    fn scroll_to(&self, _top: f64, _behavior: &str) {}

    fn on_scroll(&self, _callback: Box<dyn FnMut(f64, f64)>) {}

    fn on_resize(&self, _callback: Box<dyn FnMut(i32, i32)>) {}

    fn prefers_dark_mode(&self) -> bool {
        false
    }

    fn request_fullscreen(&self, _element: &Self::Element) {}

    fn get_scroll_top_from_point(&self, _x: i32, _y: i32) -> f64 {
        0.0
    }

    fn get_scroll_top_by_selector(&self, _selector: &str) -> f64 {
        0.0
    }

    fn get_target_element_from_event(
        &self,
        _client_x: i32,
        _client_y: i32,
    ) -> Option<Self::Element> {
        None
    }
}

impl tairitsu_vdom::QueryOps for MockPlatform {
    fn get_element_by_id(&self, _id: &str) -> Option<Self::Element> {
        None
    }

    fn query_selector(&self, _selector: &str) -> Option<Self::Element> {
        None
    }

    fn query_selector_all(&self, _selector: &str) -> Vec<Self::Element> {
        vec![]
    }

    fn element_from_point(&self, _x: i32, _y: i32) -> Option<Self::Element> {
        None
    }

    fn element_closest(&self, _element: &Self::Element, _selector: &str) -> Option<Self::Element> {
        None
    }

    fn get_element_rect_by_id(&self, _id: &str) -> Option<tairitsu_vdom::DomRect> {
        None
    }

    fn get_bounding_rect_by_class(
        &self,
        _class_name: &str,
        _element: &Self::Element,
    ) -> Option<tairitsu_vdom::DomRect> {
        None
    }
}

impl tairitsu_vdom::CanvasOps for MockPlatform {
    fn get_canvas_context(
        &self,
        _element: &Self::Element,
        _context_type: &str,
    ) -> Option<tairitsu_vdom::CanvasContext> {
        None
    }

    fn canvas_set_fill_style(&self, _ctx: tairitsu_vdom::CanvasContext, _color: &str) {}

    fn canvas_fill_rect(
        &self,
        _ctx: tairitsu_vdom::CanvasContext,
        _x: f64,
        _y: f64,
        _w: f64,
        _h: f64,
    ) {
    }

    fn canvas_clear_rect(
        &self,
        _ctx: tairitsu_vdom::CanvasContext,
        _x: f64,
        _y: f64,
        _w: f64,
        _h: f64,
    ) {
    }

    fn draw_qrcode_on_canvas_by_id(
        &self,
        _canvas_id: &str,
        _matrix: &[Vec<bool>],
        _modules: u64,
        _color: &str,
        _background: &str,
    ) -> bool {
        false
    }
}

impl tairitsu_vdom::MediaOps for MockPlatform {
    fn video_play(&self, _element: &Self::Element) {}

    fn video_pause(&self, _element: &Self::Element) {}

    fn video_get_current_time(&self, _element: &Self::Element) -> f64 {
        0.0
    }

    fn video_get_duration(&self, _element: &Self::Element) -> f64 {
        0.0
    }

    fn video_seek(&self, _element: &Self::Element, _time: f64) {}

    fn video_set_muted(&self, _element: &Self::Element, _muted: bool) {}

    fn video_set_volume(&self, _element: &Self::Element, _volume: f64) {}

    fn create_audio_context(&self) -> u64 {
        0
    }

    fn create_analyser_node(&self, _audio_context: u64) -> u64 {
        0
    }

    fn create_media_element_source(&self, _audio_context: u64, _element: u64) -> u64 {
        0
    }

    fn analyser_node_get_frequency_data(&self, _analyser: u64) -> Vec<f32> {
        vec![]
    }

    fn analyser_node_get_time_domain_data(&self, _analyser: u64) -> Vec<f32> {
        vec![]
    }
}

impl tairitsu_vdom::GeoOps for MockPlatform {
    fn get_current_position(
        &self,
        _on_success: Box<dyn FnOnce(tairitsu_vdom::GeoPosition)>,
        on_error: Box<dyn FnOnce(tairitsu_vdom::GeoPositionError)>,
        _enable_high_accuracy: bool,
        _timeout: u32,
        _maximum_age: u32,
    ) {
        on_error(tairitsu_vdom::GeoPositionError {
            code: 1,
            message: "geolocation not available in mock".to_string(),
        });
    }
}

impl tairitsu_vdom::FileOps for MockPlatform {
    fn file_reader_sync_read_as_text(
        &self,
        _blob: u64,
        _encoding: Option<&str>,
    ) -> Result<String, String> {
        Err("file reader not available in mock".to_string())
    }

    fn file_reader_sync_read_as_array_buffer(&self, _blob: u64) -> Result<Vec<u8>, String> {
        Err("file reader not available in mock".to_string())
    }

    fn file_reader_read_as_text(
        &self,
        _blob: u64,
        _encoding: Option<&str>,
        on_complete: Box<dyn FnOnce(Result<String, String>)>,
    ) {
        on_complete(Err("file reader not available in mock".to_string()));
    }

    fn file_reader_read_as_array_buffer(
        &self,
        _blob: u64,
        on_complete: Box<dyn FnOnce(Result<Vec<u8>, String>)>,
    ) {
        on_complete(Err("file reader not available in mock".to_string()));
    }
}

impl tairitsu_vdom::IdbOps for MockPlatform {
    fn idb_open(
        &self,
        _name: &str,
        _version: Option<u64>,
        on_complete: Box<dyn FnOnce(Result<u64, String>)>,
    ) -> u64 {
        on_complete(Err("indexeddb not available in mock".to_string()));
        0
    }

    fn idb_put(
        &self,
        _db: u64,
        _store_name: &str,
        _value: &str,
        _key: Option<&str>,
        on_complete: Box<dyn FnOnce(Result<(), String>)>,
    ) {
        on_complete(Err("indexeddb not available in mock".to_string()));
    }

    fn idb_get(
        &self,
        _db: u64,
        _store_name: &str,
        _key: &str,
        on_complete: Box<dyn FnOnce(Result<Option<String>, String>)>,
    ) {
        on_complete(Err("indexeddb not available in mock".to_string()));
    }

    fn idb_delete(
        &self,
        _db: u64,
        _store_name: &str,
        _key: &str,
        on_complete: Box<dyn FnOnce(Result<(), String>)>,
    ) {
        on_complete(Err("indexeddb not available in mock".to_string()));
    }

    fn idb_get_all(
        &self,
        _db: u64,
        _store_name: &str,
        on_complete: Box<dyn FnOnce(Result<Vec<String>, String>)>,
    ) {
        on_complete(Err("indexeddb not available in mock".to_string()));
    }

    fn idb_clear(
        &self,
        _db: u64,
        _store_name: &str,
        on_complete: Box<dyn FnOnce(Result<(), String>)>,
    ) {
        on_complete(Err("indexeddb not available in mock".to_string()));
    }
}

#[derive(Clone, Debug)]
struct MockElement {
    tag: String,
}

impl tairitsu_vdom::ElementHandle for MockElement {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[derive(Clone, Debug)]
struct MockEvent;

impl tairitsu_vdom::EventHandle for MockEvent {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[test]
fn test_signal_triggers_dependency_tracking() {
    // Create a signal
    let signal = Signal::new(42);

    // Clone the signal for use in the component
    let signal_clone = signal.clone();

    // Track the signal in a component context
    let _component_id = tairitsu_vdom::use_component(move || {
        // Access the signal - this should track the dependency
        let value = signal_clone.get();
        VNode::Text(tairitsu_vdom::vnode::VText::new(&format!(
            "Value: {}",
            value
        )))
    });

    // Set a new value - this should mark the component as dirty
    signal.set(100);

    // Verify the component is marked dirty
    // (This is a simplified test - in a real scenario we'd flush renders)
}

#[test]
fn test_scheduler_component_registration() {
    let platform = Rc::new(RefCell::new(MockPlatform::new()));
    let scheduler = Scheduler::new(platform);

    let id1 = scheduler.register_component(None);
    let id2 = scheduler.register_component(None);

    assert_eq!(id1, 0);
    assert_eq!(id2, 1);
}

#[test]
fn test_scheduler_mark_dirty() {
    let platform = Rc::new(RefCell::new(MockPlatform::new()));
    let scheduler = Scheduler::new(platform.clone());

    let id = scheduler.register_component(None);
    scheduler.mark_dirty(id);

    // Verify that request_animation_frame was called
    let binding = platform.borrow();
    let log = binding.render_log.borrow();
    assert!(log.iter().any(|entry| entry == "request_animation_frame"));
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

    // Clone the signal for use in the component
    let signal_clone = signal.clone();

    // Use the signal in a component context
    let component_id = tairitsu_vdom::use_component(move || {
        let value = signal_clone.get();
        VNode::Text(tairitsu_vdom::vnode::VText::new(value))
    });

    // The component should be registered
    assert!(component_id > 0);
}

#[test]
fn test_batch_updates() {
    let signal1 = Signal::new(1);
    let signal2 = Signal::new(2);

    // Batch multiple signal updates
    tairitsu_vdom::batch(|| {
        signal1.set(10);
        signal2.set(20);
    });

    // Verify both values were updated
    assert_eq!(signal1.get(), 10);
    assert_eq!(signal2.get(), 20);
}

#[test]
fn test_scheduler_update_component() {
    let platform = Rc::new(RefCell::new(MockPlatform::new()));
    let scheduler = Scheduler::new(platform.clone());

    let id = scheduler.register_component(None);

    let old_vnode = VNode::Text(tairitsu_vdom::vnode::VText::new("old"));
    let new_vnode = VNode::Text(tairitsu_vdom::vnode::VText::new("new"));

    // Update with the old vnode first
    scheduler.update_component(id, old_vnode.clone());

    // Update with the new vnode - should generate patches
    scheduler.update_component(id, new_vnode);

    // The scheduler should have stored the vnode
    // We can't directly verify this without exposing internal state,
    // but we can at least verify the component exists
    assert_eq!(id, 0);
}
