use super::r#trait::DomRect;
use crate::platform::dom::DomOps;

pub trait LayoutOps: DomOps {
    fn get_bounding_client_rect(&self, element: &Self::Element) -> DomRect;
    fn inner_width(&self) -> i32;
    fn inner_height(&self) -> i32;
    fn get_element_scroll_top(&self, element: &Self::Element) -> f64;
    fn set_element_scroll_top(&self, element: &Self::Element, value: f64);
    fn get_element_scroll_height(&self, element: &Self::Element) -> i32;
    fn get_element_client_height(&self, element: &Self::Element) -> i32;
    fn get_element_client_width(&self, element: &Self::Element) -> i32;
}
