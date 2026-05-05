use super::{ElementHandle, EventHandle};
use crate::EventData;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DomRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

pub type CanvasContext = u64;

pub trait Platform: Sized + 'static {
    type Element: ElementHandle;
    type Event: EventHandle;

    fn create_element(&self, tag: &str) -> Self::Element;
    fn create_text_node(&self, text: &str) -> Self::Element;
    fn append_child(&self, parent: &Self::Element, child: &Self::Element);
    fn remove_child(&self, parent: &Self::Element, child: &Self::Element);
    fn set_attribute(&self, element: &Self::Element, name: &str, value: &str);
    fn remove_attribute(&self, element: &Self::Element, name: &str);
    fn set_style(&self, element: &Self::Element, name: &str, value: &str);
    fn set_class(&self, element: &Self::Element, class: &str);
    fn add_event_listener(
        &self,
        element: &Self::Element,
        event: &str,
        handler: Box<dyn FnMut(Box<dyn EventData>)>,
    );
    fn remove_event_listener(&self, element: &Self::Element, event: &str);

    fn get_bounding_client_rect(&self, element: &Self::Element) -> DomRect;
    fn inner_width(&self) -> i32;
    fn inner_height(&self) -> i32;
    fn set_timeout(&self, callback: Box<dyn FnOnce()>, ms: i32) -> i32;
    fn clear_timeout(&self, id: i32);
    fn request_animation_frame(&self, callback: Box<dyn FnOnce(f64)>) -> u32;
    fn cancel_animation_frame(&self, id: u32);

    fn get_canvas_context(
        &self,
        element: &Self::Element,
        context_type: &str,
    ) -> Option<CanvasContext>;
    fn canvas_set_fill_style(&self, ctx: CanvasContext, color: &str);
    fn canvas_fill_rect(&self, ctx: CanvasContext, x: f64, y: f64, w: f64, h: f64);
    fn canvas_clear_rect(&self, ctx: CanvasContext, x: f64, y: f64, w: f64, h: f64);

    fn create_resize_observer(&self, callback: Box<dyn FnMut(Vec<ResizeObserverEntry>)>) -> u64;
    fn observe_resize(&self, observer: u64, element: &Self::Element);
    fn unobserve_resize(&self, observer: u64, element: &Self::Element);
    fn disconnect_resize(&self, observer: u64);

    fn create_mutation_observer(&self, callback: Box<dyn FnMut(Vec<MutationRecord>)>) -> u64;
    fn observe_mutations(
        &self,
        observer: u64,
        element: &Self::Element,
        options: Option<MutationObserverInit>,
    );
    fn disconnect_mutation(&self, observer: u64);

    fn match_media(&self, query: &str) -> u64;
    fn media_query_list_get_media(&self, list: u64) -> String;
    fn media_query_list_get_matches(&self, list: u64) -> bool;
    fn media_query_list_add_listener(&self, list: u64, callback: Box<dyn FnMut(bool)>) -> u64;
    fn media_query_list_remove_listener(&self, list: u64, listener_id: u64);

    fn get_element_by_id(&self, id: &str) -> Option<Self::Element>;
    fn query_selector(&self, selector: &str) -> Option<Self::Element>;
    fn query_selector_all(&self, selector: &str) -> Vec<Self::Element>;
    fn element_from_point(&self, x: i32, y: i32) -> Option<Self::Element>;
    fn element_closest(&self, element: &Self::Element, selector: &str) -> Option<Self::Element>;
    fn get_scroll_y(&self) -> f64;
    fn scroll_to(&self, top: f64, behavior: &str);
    fn on_scroll(&self, callback: Box<dyn FnMut(f64, f64)>);
    fn on_resize(&self, callback: Box<dyn FnMut(i32, i32)>);
    fn copy_to_clipboard(&self, text: &str) -> bool;
    fn read_clipboard(&self) -> Option<String>;
    fn clipboard_write_text_async(
        &self,
        text: &str,
        on_complete: Box<dyn FnOnce(Result<(), String>)>,
    );
    fn clipboard_read_text_async(&self, on_complete: Box<dyn FnOnce(Result<String, String>)>);
    fn prefers_dark_mode(&self) -> bool;
    fn get_element_rect_by_id(&self, id: &str) -> Option<DomRect>;
    fn get_bounding_rect_by_class(
        &self,
        class_name: &str,
        element: &Self::Element,
    ) -> Option<DomRect>;
    fn request_fullscreen(&self, element: &Self::Element);

    fn get_contenteditable_state(&self, element: &Self::Element) -> Option<ContentEditableState>;
    fn exec_command(&self, command: &str, value: Option<&str>) -> bool;
    fn get_selection_start(&self, element: &Self::Element) -> Option<u32>;
    fn get_selection_end(&self, element: &Self::Element) -> Option<u32>;
    fn set_content_editable(&self, element: &Self::Element, editable: bool);
    fn get_inner_html(&self, element: &Self::Element) -> String;
    fn set_inner_html(&self, element: &Self::Element, html: String);

    fn get_element_scroll_top(&self, element: &Self::Element) -> f64;
    fn set_element_scroll_top(&self, element: &Self::Element, value: f64);
    fn get_element_scroll_height(&self, element: &Self::Element) -> i32;
    fn get_element_client_height(&self, element: &Self::Element) -> i32;
    fn get_element_client_width(&self, element: &Self::Element) -> i32;

    fn get_attribute(&self, element: &Self::Element, name: &str) -> Option<String>;

    fn class_list_add(&self, element: &Self::Element, tokens: &[&str]);
    fn class_list_remove(&self, element: &Self::Element, tokens: &[&str]);
    fn class_list_contains(&self, element: &Self::Element, token: &str) -> bool;

    fn first_child(&self, element: &Self::Element) -> Option<Self::Element>;
    fn insert_before(
        &self,
        parent: &Self::Element,
        new_node: &Self::Element,
        reference_node: Option<&Self::Element>,
    );

    fn query_selector_on(&self, element: &Self::Element, selector: &str) -> Option<Self::Element>;

    fn video_play(&self, element: &Self::Element);
    fn video_pause(&self, element: &Self::Element);
    fn video_get_current_time(&self, element: &Self::Element) -> f64;
    fn video_get_duration(&self, element: &Self::Element) -> f64;
    fn video_seek(&self, element: &Self::Element, time: f64);
    fn video_set_muted(&self, element: &Self::Element, muted: bool);
    fn video_set_volume(&self, element: &Self::Element, volume: f64);
    fn create_audio_context(&self) -> u64;
    fn create_analyser_node(&self, audio_context: u64) -> u64;
    fn create_media_element_source(&self, audio_context: u64, element: u64) -> u64;
    fn analyser_node_get_frequency_data(&self, analyser: u64) -> Vec<f32>;
    fn analyser_node_get_time_domain_data(&self, analyser: u64) -> Vec<f32>;

    fn draw_qrcode_on_canvas_by_id(
        &self,
        canvas_id: &str,
        matrix: &[Vec<bool>],
        modules: u64,
        color: &str,
        background: &str,
    ) -> bool;

    fn get_scroll_top_from_point(&self, x: i32, y: i32) -> f64;
    fn get_scroll_top_by_selector(&self, selector: &str) -> f64;
    fn get_target_element_from_event(&self, client_x: i32, client_y: i32) -> Option<Self::Element>;

    fn get_current_position(
        &self,
        on_success: Box<dyn FnOnce(GeoPosition)>,
        on_error: Box<dyn FnOnce(GeoPositionError)>,
        enable_high_accuracy: bool,
        timeout: u32,
        maximum_age: u32,
    );

    // -- FileReader --

    fn file_reader_sync_read_as_text(
        &self,
        blob: u64,
        encoding: Option<&str>,
    ) -> Result<String, String>;
    fn file_reader_sync_read_as_array_buffer(&self, blob: u64) -> Result<Vec<u8>, String>;
    fn file_reader_read_as_text(
        &self,
        blob: u64,
        encoding: Option<&str>,
        on_complete: Box<dyn FnOnce(Result<String, String>)>,
    );
    fn file_reader_read_as_array_buffer(
        &self,
        blob: u64,
        on_complete: Box<dyn FnOnce(Result<Vec<u8>, String>)>,
    );

    // -- IndexedDB --

    fn idb_open(
        &self,
        name: &str,
        version: Option<u64>,
        on_complete: Box<dyn FnOnce(Result<u64, String>)>,
    ) -> u64;
    fn idb_put(
        &self,
        db: u64,
        store_name: &str,
        value: &str,
        key: Option<&str>,
        on_complete: Box<dyn FnOnce(Result<(), String>)>,
    );
    fn idb_get(
        &self,
        db: u64,
        store_name: &str,
        key: &str,
        on_complete: Box<dyn FnOnce(Result<Option<String>, String>)>,
    );
    fn idb_delete(
        &self,
        db: u64,
        store_name: &str,
        key: &str,
        on_complete: Box<dyn FnOnce(Result<(), String>)>,
    );
    fn idb_get_all(
        &self,
        db: u64,
        store_name: &str,
        on_complete: Box<dyn FnOnce(Result<Vec<String>, String>)>,
    );
    fn idb_clear(
        &self,
        db: u64,
        store_name: &str,
        on_complete: Box<dyn FnOnce(Result<(), String>)>,
    );
}

pub struct ResizeObserverEntry {
    pub target: u64,
    pub content_rect: DomRect,
    pub border_box_size: Vec<ResizeObserverSize>,
    pub content_box_size: Vec<ResizeObserverSize>,
}

pub struct ResizeObserverSize {
    pub inline_size: f64,
    pub block_size: f64,
}

pub struct MutationObserverInit {
    pub child_list: bool,
    pub attributes: bool,
    pub character_data: bool,
    pub subtree: bool,
    pub attribute_old_value: bool,
    pub character_data_old_value: bool,
}

pub struct MutationRecord {
    pub record_type: String,
    pub target: u64,
    pub added_nodes: Vec<u64>,
    pub removed_nodes: Vec<u64>,
    pub previous_sibling: Option<u64>,
    pub next_sibling: Option<u64>,
    pub attribute_name: Option<String>,
    pub attribute_namespace: Option<String>,
    pub old_value: Option<String>,
}

pub struct ContentEditableState {
    pub editable: bool,
    pub focused: bool,
}

#[derive(Clone, Debug)]
pub struct GeoPosition {
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: Option<f64>,
    pub accuracy: f64,
    pub altitude_accuracy: Option<f64>,
    pub heading: Option<f64>,
    pub speed: Option<f64>,
}

#[derive(Clone, Debug)]
pub struct GeoPositionError {
    pub code: u8,
    pub message: String,
}
