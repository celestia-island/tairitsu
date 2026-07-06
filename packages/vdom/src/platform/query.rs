use crate::platform::dom::DomOps;

pub trait QueryOps: DomOps {
    fn get_element_by_id(&self, id: &str) -> Option<Self::Element>;
    fn query_selector(&self, selector: &str) -> Option<Self::Element>;
    fn query_selector_all(&self, selector: &str) -> Vec<Self::Element>;
    fn element_from_point(&self, x: i32, y: i32) -> Option<Self::Element>;
    fn element_closest(&self, element: &Self::Element, selector: &str) -> Option<Self::Element>;
    fn get_element_rect_by_id(&self, id: &str) -> Option<super::r#trait::DomRect>;
    fn get_bounding_rect_by_class(
        &self,
        class_name: &str,
        element: &Self::Element,
    ) -> Option<super::r#trait::DomRect>;
}
