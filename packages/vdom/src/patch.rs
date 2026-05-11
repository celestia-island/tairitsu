use crate::vnode::{Classes, EventHandler, Style, VNode};

#[derive(Clone)]
pub enum Patch {
    CreateNode {
        node: VNode,
    },
    RemoveNode,
    ReplaceNode {
        node: VNode,
    },
    UpdateText {
        text: String,
    },
    UpdateAttribute {
        name: String,
        value: String,
    },
    AddAttribute {
        name: String,
        value: String,
    },
    RemoveAttribute {
        name: String,
    },
    UpdateStyle {
        style: Style,
    },
    UpdateClass {
        class: Classes,
    },
    InsertChild {
        index: usize,
        node: VNode,
    },
    RemoveChild {
        index: usize,
    },
    MoveChild {
        from: usize,
        to: usize,
    },
    UpdateChild {
        index: usize,
        patches: Vec<Patch>,
    },
    AddEvent {
        name: String,
        handler: EventHandler,
    },
    UpdateEvent {
        name: String,
        handler: EventHandler,
    },
    RemoveEvent {
        name: String,
    },
    ReorderChildren {
        removals: Vec<usize>,
        moves: Vec<(usize, usize)>,
    },
}

impl std::fmt::Debug for Patch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Patch::CreateNode { node } => f.debug_struct("CreateNode").field("node", node).finish(),
            Patch::RemoveNode => write!(f, "RemoveNode"),
            Patch::ReplaceNode { node } => {
                f.debug_struct("ReplaceNode").field("node", node).finish()
            }
            Patch::UpdateText { text } => f.debug_struct("UpdateText").field("text", text).finish(),
            Patch::UpdateAttribute { name, value } => f
                .debug_struct("UpdateAttribute")
                .field("name", name)
                .field("value", value)
                .finish(),
            Patch::AddAttribute { name, value } => f
                .debug_struct("AddAttribute")
                .field("name", name)
                .field("value", value)
                .finish(),
            Patch::RemoveAttribute { name } => f
                .debug_struct("RemoveAttribute")
                .field("name", name)
                .finish(),
            Patch::UpdateStyle { style } => {
                f.debug_struct("UpdateStyle").field("style", style).finish()
            }
            Patch::UpdateClass { class } => {
                f.debug_struct("UpdateClass").field("class", class).finish()
            }
            Patch::InsertChild { index, node } => f
                .debug_struct("InsertChild")
                .field("index", index)
                .field("node", node)
                .finish(),
            Patch::RemoveChild { index } => {
                f.debug_struct("RemoveChild").field("index", index).finish()
            }
            Patch::MoveChild { from, to } => f
                .debug_struct("MoveChild")
                .field("from", from)
                .field("to", to)
                .finish(),
            Patch::ReorderChildren { removals, moves } => f
                .debug_struct("ReorderChildren")
                .field("removals", &format!("{:?}", removals))
                .field("moves", &format!("{:?}", moves))
                .finish(),
            Patch::UpdateChild { index, patches } => f
                .debug_struct("UpdateChild")
                .field("index", index)
                .field("patches", patches)
                .finish(),
            Patch::AddEvent { name, .. } => f
                .debug_struct("AddEvent")
                .field("name", name)
                .finish_non_exhaustive(),
            Patch::UpdateEvent { name, .. } => f
                .debug_struct("UpdateEvent")
                .field("name", name)
                .finish_non_exhaustive(),
            Patch::RemoveEvent { name } => {
                f.debug_struct("RemoveEvent").field("name", name).finish()
            }
        }
    }
}

impl Patch {
    pub fn is_empty(&self) -> bool {
        matches!(self, Patch::UpdateChild { patches, .. } if patches.is_empty())
    }
}
