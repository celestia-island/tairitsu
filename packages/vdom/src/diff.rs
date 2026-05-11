use std::collections::HashMap;

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

    for (event_name, handler) in &new.event_handlers {
        if old.event_handlers.contains_key(event_name) {
            patches.push(Patch::UpdateEvent {
                name: event_name.clone(),
                handler: handler.clone(),
            });
        } else {
            patches.push(Patch::AddEvent {
                name: event_name.clone(),
                handler: handler.clone(),
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

fn extract_key(node: &VNode) -> Option<&str> {
    match node {
        VNode::Element(elem) => elem.key.as_deref(),
        _ => None,
    }
}

fn has_any_key(children: &[VNode]) -> bool {
    children.iter().any(|c| extract_key(c).is_some())
}

fn diff_children(old_children: &[VNode], new_children: &[VNode], patches: &mut Vec<Patch>) {
    let old_has_keys = has_any_key(old_children);
    let new_has_keys = has_any_key(new_children);

    if old_has_keys || new_has_keys {
        diff_children_keyed(old_children, new_children, patches);
    } else {
        diff_children_indexed(old_children, new_children, patches);
    }
}

fn diff_children_indexed(old_children: &[VNode], new_children: &[VNode], patches: &mut Vec<Patch>) {
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

fn diff_children_keyed(old_children: &[VNode], new_children: &[VNode], patches: &mut Vec<Patch>) {
    let old_len = old_children.len();
    let new_len = new_children.len();

    // Build a map from old key -> old index
    let mut old_key_map: HashMap<&str, usize> = HashMap::new();
    for (i, child) in old_children.iter().enumerate() {
        if let Some(key) = extract_key(child) {
            old_key_map.insert(key, i);
        }
    }

    // For each new child, find matching old child by key
    // new_match[i] = index in old_children, or None
    let mut new_match: Vec<Option<usize>> = Vec::with_capacity(new_len);
    for new_child in new_children.iter() {
        let matched = extract_key(new_child).and_then(|k| old_key_map.get(k).copied());
        new_match.push(matched);
    }

    // Build a set of old indices that are matched by key
    let mut matched_old_set: HashMap<usize, usize> = HashMap::new();
    for (new_idx, &old_idx_opt) in new_match.iter().enumerate() {
        if let Some(old_idx) = old_idx_opt {
            matched_old_set.insert(old_idx, new_idx);
        }
    }

    // Find LIS of matched old indices in new order
    // Collect matched old indices in the order they appear in new_children
    let matched_old_indices: Vec<usize> = new_match.iter().filter_map(|&opt| opt).collect();

    let lis_set = longest_increasing_subsequence_set(&matched_old_indices);

    // Determine which matched old indices are "stable" (part of LIS)
    // lis_set contains indices into matched_old_indices
    let mut stable_old: std::collections::HashSet<usize> = std::collections::HashSet::new();
    for &lis_idx in &lis_set {
        stable_old.insert(matched_old_indices[lis_idx]);
    }

    // Phase 1: Remove old children that are not matched
    let mut to_remove: Vec<usize> = Vec::new();
    for (i, child) in old_children.iter().enumerate() {
        if extract_key(child).is_some() && !matched_old_set.contains_key(&i) {
            to_remove.push(i);
        }
    }
    if new_has_any_key_among(new_children) {
        for (i, child) in old_children.iter().enumerate() {
            if extract_key(child).is_none() {
                to_remove.push(i);
            }
        }
    }

    // Phase 2: For each new child, generate patches
    // We track the "effective insertion index" as we go
    let mut removal_offset: isize = 0;

    // Generate removals first (in reverse order to maintain indices)
    if !to_remove.is_empty() {
        for &idx in to_remove.iter().rev() {
            patches.push(Patch::RemoveChild { index: idx });
        }
        removal_offset = to_remove.len() as isize;
    }

    // Now process new children in order
    let mut current_insert_index = old_len as isize - removal_offset;
    for (new_idx, new_child) in new_children.iter().enumerate() {
        let matched_old_idx = new_match[new_idx];

        match matched_old_idx {
            Some(old_idx) => {
                // This new child matches an old child by key
                let old_child = &old_children[old_idx];

                if stable_old.contains(&old_idx) {
                    // Stable - just update in place (adjust for removals)
                    let adjusted = old_idx as isize - count_removed_before(&to_remove, old_idx);
                    let child_patches = diff(Some(old_child), new_child);
                    if !child_patches.is_empty() {
                        patches.push(Patch::UpdateChild {
                            index: adjusted as usize,
                            patches: child_patches,
                        });
                    }
                } else {
                    // Needs to move: remove from old position, insert at new position
                    let adjusted_old = old_idx as isize - count_removed_before(&to_remove, old_idx);
                    patches.push(Patch::RemoveChild {
                        index: adjusted_old as usize,
                    });
                    patches.push(Patch::InsertChild {
                        index: current_insert_index as usize,
                        node: new_child.clone(),
                    });
                }
                current_insert_index += 1;
            }
            None => {
                // No match - insert new child
                patches.push(Patch::InsertChild {
                    index: current_insert_index as usize,
                    node: new_child.clone(),
                });
                current_insert_index += 1;
            }
        }
    }
}

fn new_has_any_key_among(children: &[VNode]) -> bool {
    children.iter().any(|c| extract_key(c).is_some())
}

fn count_removed_before(removals: &[usize], idx: usize) -> isize {
    removals.iter().filter(|&&r| r < idx).count() as isize
}

fn longest_increasing_subsequence_set(arr: &[usize]) -> Vec<usize> {
    if arr.is_empty() {
        return Vec::new();
    }

    let n = arr.len();
    let mut tails: Vec<usize> = Vec::with_capacity(n);
    let mut tail_indices: Vec<usize> = Vec::with_capacity(n);
    let mut parent: Vec<Option<usize>> = vec![None; n];

    for i in 0..n {
        let val = arr[i];

        if tails.is_empty() || val > *tails.last().unwrap() {
            if !tail_indices.is_empty() {
                parent[i] = Some(*tail_indices.last().unwrap());
            }
            tails.push(val);
            tail_indices.push(i);
        } else {
            let pos = tails.partition_point(|&x| x < val);
            tails[pos] = val;
            if pos > 0 {
                parent[i] = Some(tail_indices[pos - 1]);
            }
            tail_indices[pos] = i;
        }
    }

    // Reconstruct the LIS indices (into arr)
    let mut result = Vec::with_capacity(tail_indices.len());
    let mut current = tail_indices.last().copied();
    while let Some(idx) = current {
        result.push(idx);
        current = parent[idx];
    }
    result.reverse();
    result
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

    #[test]
    fn test_keyed_reorder_uses_move() {
        let old = vec![
            VNode::Element(
                VElement::new("li")
                    .key("a")
                    .child(VNode::Text(VText::new("A"))),
            ),
            VNode::Element(
                VElement::new("li")
                    .key("b")
                    .child(VNode::Text(VText::new("B"))),
            ),
            VNode::Element(
                VElement::new("li")
                    .key("c")
                    .child(VNode::Text(VText::new("C"))),
            ),
        ];

        let new = vec![
            VNode::Element(
                VElement::new("li")
                    .key("c")
                    .child(VNode::Text(VText::new("C"))),
            ),
            VNode::Element(
                VElement::new("li")
                    .key("a")
                    .child(VNode::Text(VText::new("A"))),
            ),
            VNode::Element(
                VElement::new("li")
                    .key("b")
                    .child(VNode::Text(VText::new("B"))),
            ),
        ];

        let mut patches = Vec::new();
        diff_children(&old, &new, &mut patches);

        // Should NOT contain ReplaceNode for any child — only moves and updates
        for p in &patches {
            if let Patch::ReplaceNode { .. } = p {
                panic!("Keyed reorder should not produce ReplaceNode patches");
            }
        }
        assert!(!patches.is_empty());
    }

    #[test]
    fn test_keyed_insert_and_remove() {
        let old = vec![
            VNode::Element(VElement::new("li").key("a")),
            VNode::Element(VElement::new("li").key("b")),
        ];

        let new = vec![
            VNode::Element(VElement::new("li").key("a")),
            VNode::Element(VElement::new("li").key("c")),
            VNode::Element(VElement::new("li").key("b")),
        ];

        let mut patches = Vec::new();
        diff_children(&old, &new, &mut patches);

        let has_insert = patches
            .iter()
            .any(|p| matches!(p, Patch::InsertChild { .. }));
        let has_remove = patches
            .iter()
            .any(|p| matches!(p, Patch::RemoveChild { .. }));
        assert!(has_insert, "Should insert new keyed child 'c'");
        assert!(!has_remove, "Should not remove any existing child");
    }

    #[test]
    fn test_unkeyed_falls_back_to_indexed() {
        let old = vec![VNode::Text(VText::new("a")), VNode::Text(VText::new("b"))];
        let new = vec![VNode::Text(VText::new("a")), VNode::Text(VText::new("c"))];

        let mut patches = Vec::new();
        diff_children(&old, &new, &mut patches);

        assert_eq!(patches.len(), 1);
        match &patches[0] {
            Patch::UpdateChild { index, .. } => assert_eq!(*index, 1),
            _ => panic!("Expected UpdateChild at index 1"),
        }
    }

    #[test]
    fn test_lis() {
        let result = longest_increasing_subsequence_set(&[3, 1, 4, 1, 5, 9, 2, 6]);
        let lis_values: Vec<usize> = result
            .iter()
            .map(|&i| [3, 1, 4, 1, 5, 9, 2, 6][i])
            .collect();
        // One valid LIS: [1, 4, 5, 9] or [1, 4, 5, 6] etc.
        for window in lis_values.windows(2) {
            assert!(window[0] < window[1], "LIS should be strictly increasing");
        }
    }
}
