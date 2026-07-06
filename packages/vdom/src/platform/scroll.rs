use crate::platform::dom::DomOps;

pub trait ScrollOps: DomOps {
    fn get_scroll_y(&self) -> f64;
    fn scroll_to(&self, top: f64, behavior: &str);
    fn on_scroll(&self, callback: Box<dyn FnMut(f64, f64)>);
    fn on_resize(&self, callback: Box<dyn FnMut(i32, i32)>);
    fn prefers_dark_mode(&self) -> bool;
    fn request_fullscreen(&self, element: &Self::Element);
    fn get_scroll_top_from_point(&self, x: i32, y: i32) -> f64;
    fn get_scroll_top_by_selector(&self, selector: &str) -> f64;
    fn get_target_element_from_event(&self, client_x: i32, client_y: i32) -> Option<Self::Element>;
}
