use super::{ElementHandle, EventHandle};

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
    fn add_event_listener(&self, element: &Self::Element, event: &str, handler: Box<dyn FnMut()>);
    fn remove_event_listener(&self, element: &Self::Element, event: &str);
}
