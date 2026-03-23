use super::{ElementHandle, EventHandle};
use crate::EventData;

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
