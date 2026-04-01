use crate::{
    patch::Patch,
    vnode::{VElement, VNode},
};

pub fn diff(old: Option<&VNode>, new: &VNode) -> Vec<Patch> {
    let mut patches = Vec::new();

    match (old, new) {
        (None, new) => {
            patches.push(Patch::CreateNode { node: new.clone() });
        }
        (Some(old), new) => {
            diff_node(old, new, &mut patches);
        }
    }

    patches
}

fn diff_node(old: &VNode, new: &VNode, patches: &mut Vec<Patch>) {
    match (old, new) {
        (VNode::Text(old_text), VNode::Text(new_text)) => {
            if old_text.text != new_text.text {
                patches.push(Patch::UpdateText {
                    text: new_text.text.clone(),
                });
            }
        }
        (VNode::Element(old_elem), VNode::Element(new_elem)) => {
            diff_element(old_elem, new_elem, patches);
        }
        (VNode::Fragment(old_children), VNode::Fragment(new_children)) => {
            diff_children(old_children, new_children, patches);
        }
        _ => {
            patches.push(Patch::ReplaceNode { node: new.clone() });
        }
    }
}

fn diff_element(old: &VElement, new: &VElement, patches: &mut Vec<Patch>) {
    if old.tag != new.tag {
        patches.push(Patch::ReplaceNode {
            node: VNode::Element(new.clone()),
        });
        return;
    }

    for (key, value) in &new.attributes {
        if let Some(old_value) = old.attributes.get(key) {
            if old_value != value {
                patches.push(Patch::UpdateAttribute {
                    name: key.clone(),
                    value: value.clone(),
                });
            }
        } else {
            patches.push(Patch::AddAttribute {
                name: key.clone(),
                value: value.clone(),
            });
        }
    }

    for key in old.attributes.keys() {
        if !new.attributes.contains_key(key) {
            patches.push(Patch::RemoveAttribute { name: key.clone() });
        }
    }

    if old.style.static_styles != new.style.static_styles {
        patches.push(Patch::UpdateStyle {
            style: new.style.clone(),
        });
    }

    if old.class.static_classes != new.class.static_classes {
        patches.push(Patch::UpdateClass {
            class: new.class.clone(),
        });
    }

    for event_name in new.event_handlers.keys() {
        if old.event_handlers.contains_key(event_name) {
            patches.push(Patch::UpdateEvent {
                name: event_name.clone(),
            });
        } else {
            patches.push(Patch::AddEvent {
                name: event_name.clone(),
            });
        }
    }

    for event_name in old.event_handlers.keys() {
        if !new.event_handlers.contains_key(event_name) {
            patches.push(Patch::RemoveEvent {
                name: event_name.clone(),
            });
        }
    }

    diff_children(&old.children, &new.children, patches);
}

fn diff_children(old_children: &[VNode], new_children: &[VNode], patches: &mut Vec<Patch>) {
    let old_len = old_children.len();
    let new_len = new_children.len();
    let max_len = old_len.max(new_len);

    for i in 0..max_len {
        let old_child = old_children.get(i);
        let new_child = new_children.get(i);

        match (old_child, new_child) {
            (Some(old), Some(new)) => {
                let child_patches = diff(Some(old), new);
                if !child_patches.is_empty() {
                    patches.push(Patch::UpdateChild {
                        index: i,
                        patches: child_patches,
                    });
                }
            }
            (None, Some(new)) => {
                patches.push(Patch::InsertChild {
                    index: i,
                    node: new.clone(),
                });
            }
            (Some(_), None) => {
                patches.push(Patch::RemoveChild { index: i });
            }
            (None, None) => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vnode::{VElement, VText};

    #[test]
    fn test_diff_text() {
        let old = VNode::Text(VText::new("hello"));
        let new = VNode::Text(VText::new("world"));

        let patches = diff(Some(&old), &new);

        assert_eq!(patches.len(), 1);
        match &patches[0] {
            Patch::UpdateText { text } => assert_eq!(text, "world"),
            _ => panic!("Expected UpdateText patch"),
        }
    }

    #[test]
    fn test_diff_element_attributes() {
        let old = VNode::Element(VElement::new("div").attr("class", "old"));
        let new = VNode::Element(VElement::new("div").attr("class", "new"));

        let patches = diff(Some(&old), &new);

        assert!(!patches.is_empty());
    }
}
