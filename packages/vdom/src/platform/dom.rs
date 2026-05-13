use super::{ElementHandle, EventHandle};
use crate::EventData;

pub trait DomOps: Sized + 'static {
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
    fn get_inner_html(&self, element: &Self::Element) -> String;
    fn set_inner_html(&self, element: &Self::Element, html: String);
}
