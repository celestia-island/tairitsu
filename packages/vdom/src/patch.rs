use crate::vnode::{Classes, Style, VNode};

#[derive(Debug, Clone)]
pub enum Patch {
    CreateNode { node: VNode },
    RemoveNode,
    ReplaceNode { node: VNode },
    UpdateText { text: String },
    UpdateAttribute { name: String, value: String },
    AddAttribute { name: String, value: String },
    RemoveAttribute { name: String },
    UpdateStyle { style: Style },
    UpdateClass { class: Classes },
    InsertChild { index: usize, node: VNode },
    RemoveChild { index: usize },
    UpdateChild { index: usize, patches: Vec<Patch> },
    AddEvent { name: String },
    UpdateEvent { name: String },
    RemoveEvent { name: String },
}

impl Patch {
    pub fn is_empty(&self) -> bool {
        matches!(self, Patch::UpdateChild { patches, .. } if patches.is_empty())
    }
}
