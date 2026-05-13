use super::r#trait::ContentEditableState;
use crate::platform::dom::DomOps;

pub trait ContentEditableOps: DomOps {
    fn get_contenteditable_state(&self, element: &Self::Element) -> Option<ContentEditableState>;
    fn exec_command(&self, command: &str, value: Option<&str>) -> bool;
    fn get_selection_start(&self, element: &Self::Element) -> Option<u32>;
    fn get_selection_end(&self, element: &Self::Element) -> Option<u32>;
    fn set_content_editable(&self, element: &Self::Element, editable: bool);
}
