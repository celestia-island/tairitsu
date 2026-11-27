use std::collections::{HashMap, HashSet};

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
        (VNode::DynamicText(old_dt), VNode::DynamicText(new_dt)) => {
            if old_dt != new_dt {
                patches.push(Patch::UpdateText {
                    text: new_dt.initial.clone(),
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
            node: VNode::Element(Box::new(new.clone())),
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

    if old.style != new.style {
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
        if let Some(old_handler) = old.event_handlers.get(event_name) {
            if !std::rc::Rc::ptr_eq(old_handler, handler) {
                patches.push(Patch::UpdateEvent {
                    name: event_name.clone(),
                    handler: handler.clone(),
                });
            }
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
                if let (VNode::DynamicText(old_dt), VNode::DynamicText(new_dt)) = (old, new) {
                    if old_dt != new_dt {
                        patches.push(Patch::UpdateChild {
                            index: i,
                            patches: vec![Patch::UpdateText {
                                text: new_dt.initial.clone(),
                            }],
                        });
                    }
                    continue;
                }
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

/// Diff children when any of them carries a key.
///
/// A keyed new child continues the old child with the same key. A new child
/// without a key continues the old child at the same rank among the unkeyed
/// children — the first with the first, the second with the second — so a
/// static sibling next to a keyed list keeps its node instead of being
/// re-created on every render.
///
/// Of the continued children, the longest run that is already in order stays
/// where it is; every other child is removed and the new one inserted at its
/// final index. Re-inserting builds a fresh DOM node, so a moved child does
/// not carry focus or selection over.
///
/// The patches are laid out to be correct in the order they are written:
///
/// 1. updates to the children that stay, against the old list, by old index;
/// 2. removals, highest old index first;
/// 3. inserts, lowest new index first.
///
/// After the removals the list holds exactly the children that stay, already
/// in their new relative order, so each insert lands at its own new index.
/// A batch without inserts also survives the applier's "updates first, then
/// removals from the top" order, which is the same sequence.
fn diff_children_keyed(old_children: &[VNode], new_children: &[VNode], patches: &mut Vec<Patch>) {
    // Old key -> old index.
    let mut old_key_map: HashMap<&str, usize> = HashMap::new();
    // Old indices of the children without a key, in order.
    let mut unkeyed_old: Vec<usize> = Vec::new();
    for (old_idx, child) in old_children.iter().enumerate() {
        match extract_key(child) {
            Some(key) => {
                old_key_map.insert(key, old_idx);
            }
            None => unkeyed_old.push(old_idx),
        }
    }

    // new_match[new_idx] = the old child the new child continues, if any.
    let mut new_match: Vec<Option<usize>> = Vec::with_capacity(new_children.len());
    let mut unkeyed_rank = 0usize;
    for new_child in new_children {
        let matched = match extract_key(new_child) {
            Some(key) => old_key_map.get(key).copied(),
            None => {
                unkeyed_rank += 1;
                unkeyed_old.get(unkeyed_rank - 1).copied()
            }
        };
        new_match.push(matched);
    }

    // The continued children, in new order: the longest run whose old indices
    // increase is the run that stays.
    let continued: Vec<usize> = new_match.iter().copied().flatten().collect();
    let mut stays: HashSet<usize> = HashSet::new();
    for &position in &longest_increasing_subsequence_set(&continued) {
        stays.insert(continued[position]);
    }

    for (new_idx, new_child) in new_children.iter().enumerate() {
        if let Some(old_idx) = new_match[new_idx] {
            if stays.contains(&old_idx) {
                let child_patches = diff(Some(&old_children[old_idx]), new_child);
                if !child_patches.is_empty() {
                    patches.push(Patch::UpdateChild {
                        index: old_idx,
                        patches: child_patches,
                    });
                }
            }
        }
    }

    for old_idx in (0..old_children.len()).rev() {
        if !stays.contains(&old_idx) {
            patches.push(Patch::RemoveChild { index: old_idx });
        }
    }

    for (new_idx, new_child) in new_children.iter().enumerate() {
        let continues_and_stays =
            new_match[new_idx].is_some_and(|old_idx| stays.contains(&old_idx));
        if !continues_and_stays {
            patches.push(Patch::InsertChild {
                index: new_idx,
                node: new_child.clone(),
            });
        }
    }
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

        if tails.is_empty()
            || val
                > *tails
                    .last()
                    .expect("tails is non-empty (invariant: checked is_empty above)")
        {
            if !tail_indices.is_empty() {
                parent[i] = Some(
                    *tail_indices
                        .last()
                        .expect("tail_indices is non-empty (invariant: checked is_empty above)"),
                );
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
    use crate::vnode::{Classes, Style, VElement, VText};

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
        let old = VNode::Element(Box::new(VElement::new("div").attr("class", "old")));
        let new = VNode::Element(Box::new(VElement::new("div").attr("class", "new")));

        let patches = diff(Some(&old), &new);

        assert!(!patches.is_empty());
    }

    #[test]
    fn test_keyed_reorder_uses_move() {
        let old = vec![
            VNode::Element(Box::new(
                VElement::new("li")
                    .key("a")
                    .child(VNode::Text(VText::new("A"))),
            )),
            VNode::Element(Box::new(
                VElement::new("li")
                    .key("b")
                    .child(VNode::Text(VText::new("B"))),
            )),
            VNode::Element(Box::new(
                VElement::new("li")
                    .key("c")
                    .child(VNode::Text(VText::new("C"))),
            )),
        ];

        let new = vec![
            VNode::Element(Box::new(
                VElement::new("li")
                    .key("c")
                    .child(VNode::Text(VText::new("C"))),
            )),
            VNode::Element(Box::new(
                VElement::new("li")
                    .key("a")
                    .child(VNode::Text(VText::new("A"))),
            )),
            VNode::Element(Box::new(
                VElement::new("li")
                    .key("b")
                    .child(VNode::Text(VText::new("B"))),
            )),
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
            VNode::Element(Box::new(VElement::new("li").key("a"))),
            VNode::Element(Box::new(VElement::new("li").key("b"))),
        ];

        let new = vec![
            VNode::Element(Box::new(VElement::new("li").key("a"))),
            VNode::Element(Box::new(VElement::new("li").key("c"))),
            VNode::Element(Box::new(VElement::new("li").key("b"))),
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

    // -- Keyed children, checked by the order the patches leave behind --------
    //
    // The tests above only look at which kinds of patch come out. These run
    // the patches, in the order they are written, against a list of labels
    // and compare the result with the new order — which is the only thing
    // the DOM will see.

    /// What a child is called in the model: its key, or its text.
    fn label(node: &VNode) -> String {
        match node {
            VNode::Element(elem) => elem.key.clone().unwrap_or_else(|| "li".to_string()),
            VNode::Text(text) => text.text.clone(),
            _ => String::from("node"),
        }
    }

    /// Apply a sibling batch to a list of labels, in the order it is written.
    /// An insert past the end lands at the end, as the DOM applier does.
    fn apply_in_written_order(list: &mut Vec<String>, patches: &[Patch]) {
        for patch in patches {
            match patch {
                Patch::InsertChild { index, node } => {
                    let at = (*index).min(list.len());
                    list.insert(at, label(node));
                }
                Patch::RemoveChild { index } => {
                    if *index < list.len() {
                        list.remove(*index);
                    }
                }
                Patch::UpdateChild { index, patches } => {
                    for nested in patches {
                        match nested {
                            Patch::UpdateText { text } => list[*index] = text.clone(),
                            Patch::ReplaceNode { node } => list[*index] = label(node),
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }
    }

    /// A keyed `<li>` whose text is its own key.
    fn keyed(keys: &[&str]) -> Vec<VNode> {
        keys.iter()
            .map(|key| {
                VNode::Element(Box::new(
                    VElement::new("li")
                        .key(key)
                        .child(VNode::Text(VText::new(key))),
                ))
            })
            .collect()
    }

    fn unkeyed(text: &str) -> VNode {
        VNode::Text(VText::new(text))
    }

    /// Diff `old` into `new` and return the list the written patches leave.
    fn rewritten(old: Vec<VNode>, new: &[VNode]) -> Vec<String> {
        let mut list: Vec<String> = old.iter().map(label).collect();
        let mut patches = Vec::new();
        diff_children(&old, new, &mut patches);
        apply_in_written_order(&mut list, &patches);
        list
    }

    fn rewritten_keys(old: &[&str], new: &[&str]) -> Vec<String> {
        rewritten(keyed(old), &keyed(new))
    }

    fn expected(keys: &[&str]) -> Vec<String> {
        keys.iter().map(|key| key.to_string()).collect()
    }

    #[test]
    fn keyed_children_insert_anywhere() {
        for new in [["x", "a", "b"], ["a", "x", "b"], ["a", "b", "x"]] {
            assert_eq!(
                rewritten_keys(&["a", "b"], &new),
                expected(&new),
                "inserted as {new:?}"
            );
        }
    }

    #[test]
    fn keyed_children_remove_any_where() {
        // Every subset of [a, b, c, d], the empty one included.
        for mask in 0..16u32 {
            let keep: Vec<&str> = ["a", "b", "c", "d"]
                .iter()
                .enumerate()
                .filter(|(i, _)| mask & (1 << i) != 0)
                .map(|(_, key)| *key)
                .collect();
            assert_eq!(
                rewritten_keys(&["a", "b", "c", "d"], &keep),
                expected(&keep),
                "keeping {keep:?}"
            );
        }
    }

    fn permutations(keys: &[&'static str]) -> Vec<Vec<&'static str>> {
        if keys.len() <= 1 {
            return vec![keys.to_vec()];
        }
        let mut out = Vec::new();
        for (i, first) in keys.iter().enumerate() {
            let mut rest = keys.to_vec();
            rest.remove(i);
            for tail in permutations(&rest) {
                out.push(vec![*first].into_iter().chain(tail).collect());
            }
        }
        out
    }

    #[test]
    fn keyed_children_every_order_lands_in_the_new_order() {
        let all = permutations(&["a", "b", "c", "d"]);
        assert_eq!(all.len(), 24);
        for new in all {
            assert_eq!(
                rewritten_keys(&["a", "b", "c", "d"], &new),
                expected(&new),
                "reordered as {new:?}"
            );
        }
    }

    #[test]
    fn keyed_children_remove_insert_and_move_in_one_batch() {
        assert_eq!(
            rewritten_keys(&["a", "b", "c", "d"], &["d", "x", "b"]),
            expected(&["d", "x", "b"])
        );
        assert_eq!(
            rewritten_keys(&["a", "b", "c"], &["c", "y", "a", "z"]),
            expected(&["c", "y", "a", "z"])
        );
    }

    #[test]
    fn keyed_children_grow_from_and_shrink_to_nothing() {
        assert_eq!(rewritten_keys(&[], &["a", "b"]), expected(&["a", "b"]));
        assert_eq!(rewritten_keys(&["a", "b"], &[]), Vec::<String>::new());
        assert_eq!(
            rewritten_keys(&["a", "b"], &["x", "y"]),
            expected(&["x", "y"])
        );
    }

    #[test]
    fn keyed_children_update_the_ones_that_stay() {
        let old = vec![
            VNode::Element(Box::new(
                VElement::new("li")
                    .key("a")
                    .child(VNode::Text(VText::new("A"))),
            )),
            VNode::Element(Box::new(
                VElement::new("li")
                    .key("b")
                    .child(VNode::Text(VText::new("B"))),
            )),
        ];
        // `a` changes, and `x` arrives in front of it: the update must still
        // find `a`, and `a` must end up second.
        let new = vec![
            VNode::Element(Box::new(VElement::new("li").key("x").child(unkeyed("X")))),
            VNode::Element(Box::new(
                VElement::new("li")
                    .key("a")
                    .child(VNode::Text(VText::new("A2"))),
            )),
            VNode::Element(Box::new(
                VElement::new("li")
                    .key("b")
                    .child(VNode::Text(VText::new("B"))),
            )),
        ];
        let mut list: Vec<String> = old.iter().map(label).collect();
        let mut patches = Vec::new();
        diff_children(&old, &new, &mut patches);
        apply_in_written_order(&mut list, &patches);
        assert_eq!(list, vec!["x", "a", "b"]);

        // The text change rides an UpdateChild addressed at `a`'s old index,
        // and `b`, which did not change, gets no update at all.
        let updated: Vec<usize> = patches
            .iter()
            .filter_map(|patch| match patch {
                Patch::UpdateChild { index, .. } => Some(*index),
                _ => None,
            })
            .collect();
        assert_eq!(
            updated,
            vec![0],
            "a sits at index 0 until the batch moves it: {:?}",
            patches
        );
    }

    #[test]
    fn unkeyed_siblings_keep_their_node_beside_keyed_children() {
        let mut old = vec![unkeyed("Title")];
        old.extend(keyed(&["a", "b"]));
        let mut new = vec![unkeyed("Title")];
        new.extend(keyed(&["b", "a"]));
        assert_eq!(rewritten(old, &new), vec!["Title", "b", "a"]);

        // A changed static sibling is updated in place, not duplicated.
        let mut old = vec![unkeyed("Title")];
        old.extend(keyed(&["a"]));
        let mut new = vec![unkeyed("TITLE!")];
        new.extend(keyed(&["a"]));
        assert_eq!(rewritten(old, &new), vec!["TITLE!", "a"]);

        let mut old = keyed(&["a"]);
        old.push(unkeyed("sep"));
        old.extend(keyed(&["b"]));
        let mut new = keyed(&["b"]);
        new.push(unkeyed("sep"));
        new.extend(keyed(&["a"]));
        assert_eq!(rewritten(old, &new), vec!["b", "sep", "a"]);
    }

    #[test]
    fn unkeyed_siblings_match_by_rank() {
        // Old unkeyed children are [x]; new ones are [y, z]. The first new one
        // continues x's node wherever it now sits, the second is new.
        let mut old = keyed(&["a"]);
        old.push(unkeyed("x"));
        let mut new = vec![unkeyed("y")];
        new.extend(keyed(&["a"]));
        new.push(unkeyed("z"));
        assert_eq!(rewritten(old, &new), vec!["y", "a", "z"]);

        // Both unkeyed children keep their node and take the new text.
        let mut old = vec![unkeyed("p")];
        old.extend(keyed(&["a"]));
        old.push(unkeyed("q"));
        let mut new = vec![unkeyed("p2")];
        new.extend(keyed(&["a"]));
        new.push(unkeyed("q2"));
        assert_eq!(rewritten(old, &new), vec!["p2", "a", "q2"]);
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

    #[test]
    fn test_diff_element_different_tag_replaces() {
        let old = VNode::Element(Box::new(VElement::new("div")));
        let new = VNode::Element(Box::new(VElement::new("span")));

        let patches = diff(Some(&old), &new);

        assert_eq!(patches.len(), 1);
        match &patches[0] {
            Patch::ReplaceNode { node } => match node {
                VNode::Element(elem) => assert_eq!(elem.tag, "span"),
                _ => panic!("Expected Element node in ReplaceNode"),
            },
            _ => panic!("Expected ReplaceNode, got {:?}", patches[0]),
        }
    }

    #[test]
    fn test_diff_element_attribute_addition() {
        let old = VNode::Element(Box::new(VElement::new("div")));
        let new = VNode::Element(Box::new(
            VElement::new("div").attr("id", "app").attr("data-x", "1"),
        ));

        let patches = diff(Some(&old), &new);

        let add_attrs: Vec<_> = patches
            .iter()
            .filter_map(|p| match p {
                Patch::AddAttribute { name, value } => Some((name.clone(), value.clone())),
                _ => None,
            })
            .collect();

        assert_eq!(add_attrs.len(), 2);
        assert!(
            add_attrs.contains(&("id".to_string(), "app".to_string())),
            "should add id attribute"
        );
        assert!(
            add_attrs.contains(&("data-x".to_string(), "1".to_string())),
            "should add data-x attribute"
        );
    }

    #[test]
    fn test_diff_element_attribute_removal() {
        let old = VNode::Element(Box::new(
            VElement::new("div")
                .attr("id", "app")
                .attr("data-x", "1")
                .attr("role", "main"),
        ));
        let new = VNode::Element(Box::new(VElement::new("div").attr("id", "app")));

        let patches = diff(Some(&old), &new);

        let removed: Vec<_> = patches
            .iter()
            .filter_map(|p| match p {
                Patch::RemoveAttribute { name } => Some(name.clone()),
                _ => None,
            })
            .collect();

        assert_eq!(removed.len(), 2);
        assert!(removed.iter().any(|name| name == "data-x"));
        assert!(removed.iter().any(|name| name == "role"));
    }

    #[test]
    fn test_diff_element_attribute_update() {
        let old = VNode::Element(Box::new(
            VElement::new("div").attr("class", "old").attr("id", "x"),
        ));
        let new = VNode::Element(Box::new(
            VElement::new("div").attr("class", "new").attr("id", "x"),
        ));

        let patches = diff(Some(&old), &new);

        let updated: Vec<_> = patches
            .iter()
            .filter_map(|p| match p {
                Patch::UpdateAttribute { name, value } => Some((name.clone(), value.clone())),
                _ => None,
            })
            .collect();

        assert_eq!(updated.len(), 1);
        assert_eq!(updated[0], ("class".to_string(), "new".to_string()));
    }

    #[test]
    fn test_diff_element_style_changes() {
        let old = VNode::Element(Box::new(
            VElement::new("div").style(Style::new().add("color", "red")),
        ));
        let new = VNode::Element(Box::new(
            VElement::new("div").style(Style::new().add("color", "blue").add("font-size", "16px")),
        ));

        let patches = diff(Some(&old), &new);

        let style_patches: Vec<_> = patches
            .iter()
            .filter(|p| matches!(p, Patch::UpdateStyle { .. }))
            .collect();

        assert_eq!(style_patches.len(), 1);
        match &style_patches[0] {
            Patch::UpdateStyle { style } => {
                assert!(style.static_styles.contains("color:blue"));
                assert!(style.static_styles.contains("font-size:16px"));
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn test_diff_element_style_no_change() {
        let old = VNode::Element(Box::new(
            VElement::new("div").style(Style::new().add("color", "red")),
        ));
        let new = VNode::Element(Box::new(
            VElement::new("div").style(Style::new().add("color", "red")),
        ));

        let patches = diff(Some(&old), &new);

        assert!(
            !patches
                .iter()
                .any(|p| matches!(p, Patch::UpdateStyle { .. })),
            "no UpdateStyle when styles are identical"
        );
    }

    #[test]
    fn test_diff_element_class_changes() {
        let old = VNode::Element(Box::new(
            VElement::new("div").class(Classes::new().add("a").add("b")),
        ));
        let new = VNode::Element(Box::new(
            VElement::new("div").class(Classes::new().add("a").add("c")),
        ));

        let patches = diff(Some(&old), &new);

        let class_patches: Vec<_> = patches
            .iter()
            .filter(|p| matches!(p, Patch::UpdateClass { .. }))
            .collect();

        assert_eq!(class_patches.len(), 1);
        match &class_patches[0] {
            Patch::UpdateClass { class } => {
                assert_eq!(class.static_classes, "a c");
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn test_diff_element_class_no_change() {
        let old = VNode::Element(Box::new(
            VElement::new("div").class(Classes::new().add("foo")),
        ));
        let new = VNode::Element(Box::new(
            VElement::new("div").class(Classes::new().add("foo")),
        ));

        let patches = diff(Some(&old), &new);

        assert!(
            !patches
                .iter()
                .any(|p| matches!(p, Patch::UpdateClass { .. })),
            "no UpdateClass when classes are identical"
        );
    }

    #[test]
    fn test_diff_children_mixed_keyed_unkeyed() {
        let old = vec![
            VNode::Element(Box::new(VElement::new("li").key("a"))),
            VNode::Text(VText::new("unkeyed")),
            VNode::Element(Box::new(VElement::new("li").key("b"))),
        ];

        let new = vec![
            VNode::Element(Box::new(VElement::new("li").key("b"))),
            VNode::Text(VText::new("unkeyed-changed")),
        ];

        let mut patches = Vec::new();
        diff_children(&old, &new, &mut patches);

        assert!(
            !patches.is_empty(),
            "mixing keyed and unkeyed should produce patches"
        );
    }

    #[test]
    fn test_diff_children_remove_all() {
        let old = vec![
            VNode::Text(VText::new("a")),
            VNode::Text(VText::new("b")),
            VNode::Text(VText::new("c")),
        ];
        let new: Vec<VNode> = vec![];

        let mut patches = Vec::new();
        diff_children(&old, &new, &mut patches);

        let removes: Vec<_> = patches
            .iter()
            .filter_map(|p| match p {
                Patch::RemoveChild { index } => Some(*index),
                _ => None,
            })
            .collect();

        assert_eq!(removes.len(), 3);
        assert!(removes.contains(&0));
        assert!(removes.contains(&1));
        assert!(removes.contains(&2));
    }

    #[test]
    fn test_diff_children_replace_all() {
        let old = vec![VNode::Text(VText::new("a")), VNode::Text(VText::new("b"))];
        let new = vec![
            VNode::Element(Box::new(VElement::new("span"))),
            VNode::Element(Box::new(VElement::new("div"))),
        ];

        let mut patches = Vec::new();
        diff_children(&old, &new, &mut patches);

        let updates: Vec<_> = patches
            .iter()
            .filter_map(|p| match p {
                Patch::UpdateChild { index, .. } => Some(*index),
                _ => None,
            })
            .collect();

        assert_eq!(updates.len(), 2, "both children should be updated");
        assert!(updates.contains(&0));
        assert!(updates.contains(&1));

        for p in &patches {
            if let Patch::UpdateChild {
                index,
                patches: child_patches,
            } = p
            {
                assert!(
                    !child_patches.is_empty(),
                    "child {} should have patches",
                    index
                );
                match index {
                    0 => assert!(
                        child_patches
                            .iter()
                            .any(|cp| matches!(cp, Patch::ReplaceNode { .. })),
                        "text->element should be ReplaceNode"
                    ),
                    1 => assert!(
                        child_patches
                            .iter()
                            .any(|cp| matches!(cp, Patch::ReplaceNode { .. })),
                        "text->element should be ReplaceNode"
                    ),
                    _ => {}
                }
            }
        }
    }

    #[test]
    fn test_diff_none_creates_node() {
        let new = VNode::Text(VText::new("hello"));
        let patches = diff(None, &new);

        assert_eq!(patches.len(), 1);
        match &patches[0] {
            Patch::CreateNode { node } => match node {
                VNode::Text(t) => assert_eq!(t.text, "hello"),
                _ => panic!("Expected text node"),
            },
            _ => panic!("Expected CreateNode"),
        }
    }

    #[test]
    fn test_diff_same_text_no_patches() {
        let old = VNode::Text(VText::new("same"));
        let new = VNode::Text(VText::new("same"));

        let patches = diff(Some(&old), &new);
        assert!(patches.is_empty());
    }

    #[test]
    fn test_diff_type_change_replaces() {
        let old = VNode::Text(VText::new("text"));
        let new = VNode::Element(Box::new(VElement::new("div")));

        let patches = diff(Some(&old), &new);
        assert_eq!(patches.len(), 1);
        assert!(matches!(&patches[0], Patch::ReplaceNode { .. }));
    }

    #[test]
    fn test_diff_dynamic_text_unchanged() {
        let old = VNode::DynamicText(crate::vnode::DynamicText::new("same".into(), || {
            "same".to_string()
        }));
        let new = VNode::DynamicText(crate::vnode::DynamicText::new("same".into(), || {
            "same".to_string()
        }));

        let patches = diff(Some(&old), &new);
        assert!(
            patches.is_empty(),
            "Identical DynamicText should produce no patches"
        );
    }

    #[test]
    fn test_diff_dynamic_text_changed() {
        let old = VNode::DynamicText(crate::vnode::DynamicText::new("hello".into(), || {
            "hello".to_string()
        }));
        let new = VNode::DynamicText(crate::vnode::DynamicText::new("world".into(), || {
            "world".to_string()
        }));

        let patches = diff(Some(&old), &new);
        assert_eq!(patches.len(), 1);
        assert!(matches!(&patches[0], Patch::UpdateText { text } if text == "world"));
    }

    #[test]
    fn test_diff_dynamic_text_to_text_replaces() {
        let old = VNode::DynamicText(crate::vnode::DynamicText::new("hello".into(), || {
            "hello".into()
        }));
        let new = VNode::Text(VText::new("world"));

        let patches = diff(Some(&old), &new);
        assert_eq!(patches.len(), 1);
        assert!(matches!(&patches[0], Patch::ReplaceNode { .. }));
    }

    #[test]
    fn test_diff_text_to_dynamic_text_replaces() {
        let old = VNode::Text(VText::new("hello"));
        let new = VNode::DynamicText(crate::vnode::DynamicText::new("hello".into(), || {
            "hello".into()
        }));

        let patches = diff(Some(&old), &new);
        assert_eq!(patches.len(), 1);
        assert!(matches!(&patches[0], Patch::ReplaceNode { .. }));
    }

    #[test]
    fn test_dynamic_text_in_children_changed() {
        let old = VNode::Element(Box::new(VElement::new("div").child(VNode::DynamicText(
            crate::vnode::DynamicText::new("v1".into(), || "v1".into()),
        ))));
        let new = VNode::Element(Box::new(VElement::new("div").child(VNode::DynamicText(
            crate::vnode::DynamicText::new("v2".into(), || "v2".into()),
        ))));

        let patches = diff(Some(&old), &new);
        assert!(!patches.is_empty());

        // Should contain an UpdateChild with an UpdateText patch inside
        let has_update = patches.iter().any(|p| match p {
            Patch::UpdateChild {
                index,
                patches: child_patches,
            } => {
                *index == 0
                    && child_patches
                        .iter()
                        .any(|cp| matches!(cp, Patch::UpdateText { text } if text == "v2"))
            }
            _ => false,
        });
        assert!(
            has_update,
            "Expected UpdateChild with UpdateText for changed DynamicText"
        );
    }

    #[test]
    fn test_diff_fragment_children() {
        let old = VNode::Fragment(vec![
            VNode::Text(VText::new("a")),
            VNode::Text(VText::new("b")),
        ]);
        let new = VNode::Fragment(vec![
            VNode::Text(VText::new("a")),
            VNode::Text(VText::new("c")),
        ]);

        let patches = diff(Some(&old), &new);
        assert!(!patches.is_empty());
        match &patches[0] {
            Patch::UpdateChild { index: 1, .. } => {}
            other => panic!("Expected UpdateChild at index 1, got {:?}", other),
        }
    }

    #[test]
    fn test_diff_fragment_grow() {
        let old = VNode::Fragment(vec![VNode::Text(VText::new("a"))]);
        let new = VNode::Fragment(vec![
            VNode::Text(VText::new("a")),
            VNode::Text(VText::new("b")),
        ]);

        let patches = diff(Some(&old), &new);
        assert!(!patches.is_empty());
        assert!(patches
            .iter()
            .any(|p| matches!(p, Patch::InsertChild { index: 1, .. })));
    }

    #[test]
    fn test_diff_fragment_shrink() {
        let old = VNode::Fragment(vec![
            VNode::Text(VText::new("a")),
            VNode::Text(VText::new("b")),
        ]);
        let new = VNode::Fragment(vec![VNode::Text(VText::new("a"))]);

        let patches = diff(Some(&old), &new);
        assert!(!patches.is_empty());
        assert!(patches
            .iter()
            .any(|p| matches!(p, Patch::RemoveChild { index: 1 })));
    }

    #[test]
    fn test_diff_css_variable_change() {
        let old = VNode::Element(Box::new(
            VElement::new("div").style(Style::new().add_custom("--color", "red")),
        ));
        let new = VNode::Element(Box::new(
            VElement::new("div").style(
                Style::new()
                    .add_custom("--color", "blue")
                    .add("color", "var(--color)"),
            ),
        ));

        let patches = diff(Some(&old), &new);
        assert!(
            !patches.is_empty(),
            "Style with different static_styles should produce patches"
        );
    }

    #[test]
    fn test_diff_css_variable_only_change() {
        let old = VNode::Element(Box::new(
            VElement::new("div").style(Style::new().add_custom("--gap", "8px")),
        ));
        let new = VNode::Element(Box::new(
            VElement::new("div").style(Style::new().add_custom("--gap", "16px")),
        ));

        let patches = diff(Some(&old), &new);
        assert!(
            !patches.is_empty(),
            "Changing only a CSS variable (no static_styles change) should produce UpdateStyle patch"
        );
    }

    #[test]
    fn test_diff_event_handler_change_produces_update() {
        let old = VNode::Element(Box::new(
            VElement::new("button").on_event("click", move |_| {}),
        ));
        let new = VNode::Element(Box::new(
            VElement::new("button").on_event("click", move |_| {}),
        ));

        let patches = diff(Some(&old), &new);
        assert!(
            patches
                .iter()
                .any(|p| matches!(p, Patch::UpdateEvent { .. })),
            "Re-registered event should produce UpdateEvent"
        );
    }

    #[test]
    fn test_diff_event_handler_add() {
        let old = VNode::Element(Box::new(VElement::new("button")));
        let new = VNode::Element(Box::new(
            VElement::new("button").on_event("click", move |_| {}),
        ));

        let patches = diff(Some(&old), &new);
        assert!(patches.iter().any(|p| matches!(p, Patch::AddEvent { .. })));
    }

    #[test]
    fn test_diff_event_handler_remove() {
        let old = VNode::Element(Box::new(
            VElement::new("button").on_event("click", move |_| {}),
        ));
        let new = VNode::Element(Box::new(VElement::new("button")));

        let patches = diff(Some(&old), &new);
        assert!(patches
            .iter()
            .any(|p| matches!(p, Patch::RemoveEvent { .. })));
    }

    #[test]
    fn test_diff_event_handler_change() {
        let old = VNode::Element(Box::new(
            VElement::new("button")
                .on_event("click", |_| {})
                .on_event("mouseover", |_| {}),
        ));
        let new = VNode::Element(Box::new(
            VElement::new("button")
                .on_event("click", |_| {})
                .on_event("mouseout", |_| {}),
        ));

        let patches = diff(Some(&old), &new);
        assert!(patches
            .iter()
            .any(|p| matches!(p, Patch::RemoveEvent { .. })));
        assert!(patches.iter().any(|p| matches!(p, Patch::AddEvent { .. })));
    }

    #[test]
    fn test_diff_event_handler_unchanged_produces_update() {
        let old = VNode::Element(Box::new(
            VElement::new("button").on_event("click", move |_| {}),
        ));
        let new = VNode::Element(Box::new(
            VElement::new("button").on_event("click", move |_| {}),
        ));

        let patches = diff(Some(&old), &new);
        assert!(
            patches
                .iter()
                .any(|p| matches!(p, Patch::UpdateEvent { .. })),
            "Same event key produces UpdateEvent (handlers are not compared by value)"
        );
    }

    #[test]
    fn test_diff_nested_element_children() {
        let old = VNode::Element(Box::new(VElement::new("div").child(VNode::Element(
            Box::new(VElement::new("span").child(VNode::Text(VText::new("old")))),
        ))));
        let new = VNode::Element(Box::new(VElement::new("div").child(VNode::Element(
            Box::new(VElement::new("span").child(VNode::Text(VText::new("new")))),
        ))));

        let patches = diff(Some(&old), &new);
        assert!(!patches.is_empty());
        match &patches[0] {
            Patch::UpdateChild { index: 0, patches } => {
                assert!(patches
                    .iter()
                    .any(|p| matches!(p, Patch::UpdateChild { .. })));
            }
            other => panic!("Expected UpdateChild at index 0, got {:?}", other),
        }
    }

    #[test]
    fn test_diff_keyed_with_same_keys_different_order() {
        let old = VNode::Element(Box::new(VElement::new("ul").children(vec![
                VNode::Element(Box::new(
                    VElement::new("li")
                        .key("a")
                        .child(VNode::Text(VText::new("A"))),
                )),
                VNode::Element(Box::new(
                    VElement::new("li")
                        .key("b")
                        .child(VNode::Text(VText::new("B"))),
                )),
                VNode::Element(Box::new(
                    VElement::new("li")
                        .key("c")
                        .child(VNode::Text(VText::new("C"))),
                )),
            ])));
        let new = VNode::Element(Box::new(VElement::new("ul").children(vec![
                VNode::Element(Box::new(
                    VElement::new("li")
                        .key("c")
                        .child(VNode::Text(VText::new("C"))),
                )),
                VNode::Element(Box::new(
                    VElement::new("li")
                        .key("a")
                        .child(VNode::Text(VText::new("A"))),
                )),
                VNode::Element(Box::new(
                    VElement::new("li")
                        .key("b")
                        .child(VNode::Text(VText::new("B"))),
                )),
            ])));

        let patches = diff(Some(&old), &new);
        assert!(!patches.is_empty(), "Keyed reorder should produce patches");
        assert!(
            patches
                .iter()
                .any(|p| matches!(p, Patch::RemoveChild { .. }))
                || patches
                    .iter()
                    .any(|p| matches!(p, Patch::InsertChild { .. })),
            "Keyed reorder should produce Remove/Insert patches"
        );
    }

    #[test]
    fn test_diff_empty_fragment_to_children() {
        let old = VNode::Fragment(vec![]);
        let new = VNode::Fragment(vec![VNode::Text(VText::new("hello"))]);

        let patches = diff(Some(&old), &new);
        assert_eq!(patches.len(), 1);
        assert!(matches!(&patches[0], Patch::InsertChild { index: 0, .. }));
    }

    #[test]
    fn test_diff_none_to_element() {
        let new = VNode::Element(Box::new(VElement::new("div")));

        let patches = diff(None, &new);
        assert_eq!(patches.len(), 1);
        assert!(matches!(&patches[0], Patch::CreateNode { .. }));
    }

    #[test]
    fn test_diff_mixed_keyed_preserves_unkeyed() {
        // Old: [A(unkeyed), B(key="b"), C(unkeyed)]
        // New: [B(key="b"), A(unkeyed), C(unkeyed)]
        // Unkeyed children should NOT be destroyed when keyed children exist.
        let old_children = vec![
            VNode::Text(VText::new("A")),
            VNode::Element(Box::new(
                VElement::new("span")
                    .key("b")
                    .child(VNode::Text(VText::new("B"))),
            )),
            VNode::Text(VText::new("C")),
        ];
        let new_children = vec![
            VNode::Element(Box::new(
                VElement::new("span")
                    .key("b")
                    .child(VNode::Text(VText::new("B"))),
            )),
            VNode::Text(VText::new("A")),
            VNode::Text(VText::new("C")),
        ];

        let patches = diff(
            Some(&VNode::Fragment(old_children)),
            &VNode::Fragment(new_children),
        );

        // Neither unkeyed child is destroyed: the only child taken out is the
        // keyed one, which moved from index 1 to index 0 and is re-inserted
        // there. The old run left the unkeyed children alone by never
        // removing anything, and inserted a second copy of each instead.
        let removes: Vec<usize> = patches
            .iter()
            .filter_map(|p| match p {
                Patch::RemoveChild { index } => Some(*index),
                _ => None,
            })
            .collect();
        assert_eq!(
            removes,
            vec![1],
            "only the keyed child moves: {:?}",
            patches
        );
        let mut list = vec!["A".to_string(), "b".to_string(), "C".to_string()];
        apply_in_written_order(&mut list, &patches);
        assert_eq!(list, vec!["b", "A", "C"]);
    }

    #[test]
    fn test_diff_mixed_keyed_remove_keyed_only() {
        // Old: [A(key="a"), B(key="b"), C(unkeyed)]
        // New: [B(key="b"), C(unkeyed)]
        // Only A (keyed and removed) should be removed, not C.
        let old_children = vec![
            VNode::Element(Box::new(VElement::new("span").key("a"))),
            VNode::Element(Box::new(VElement::new("span").key("b"))),
            VNode::Text(VText::new("C")),
        ];
        let new_children = vec![
            VNode::Element(Box::new(VElement::new("span").key("b"))),
            VNode::Text(VText::new("C")),
        ];

        let patches = diff(
            Some(&VNode::Fragment(old_children)),
            &VNode::Fragment(new_children),
        );

        // Should remove only A (index 0, keyed, removed from new)
        let removes: Vec<&Patch> = patches
            .iter()
            .filter(|p| matches!(p, Patch::RemoveChild { .. }))
            .collect();
        assert_eq!(removes.len(), 1, "should remove exactly one child");
        assert!(matches!(removes[0], Patch::RemoveChild { index: 0 }));
    }

    #[test]
    fn test_diff_keyed_all_reordered() {
        // Old: [A(key="a"), B(key="b"), C(key="c")]
        // New: [C(key="c"), A(key="a"), B(key="b")]
        let old = vec![
            VNode::Element(Box::new(VElement::new("li").key("a"))),
            VNode::Element(Box::new(VElement::new("li").key("b"))),
            VNode::Element(Box::new(VElement::new("li").key("c"))),
        ];
        let new = vec![
            VNode::Element(Box::new(VElement::new("li").key("c"))),
            VNode::Element(Box::new(VElement::new("li").key("a"))),
            VNode::Element(Box::new(VElement::new("li").key("b"))),
        ];
        let patches = diff(Some(&VNode::Fragment(old)), &VNode::Fragment(new));
        assert!(
            patches
                .iter()
                .any(|p| matches!(p, Patch::RemoveChild { .. } | Patch::InsertChild { .. })),
            "reorder should produce remove+insert patches"
        );
    }

    #[test]
    fn test_diff_keyed_add_and_remove() {
        // Old: [A, B, C] (all keyed)
        // New: [D, B]     (A/C removed, D added)
        let old = vec![
            VNode::Element(Box::new(VElement::new("li").key("a"))),
            VNode::Element(Box::new(VElement::new("li").key("b"))),
            VNode::Element(Box::new(VElement::new("li").key("c"))),
        ];
        let new = vec![
            VNode::Element(Box::new(VElement::new("li").key("d"))),
            VNode::Element(Box::new(VElement::new("li").key("b"))),
        ];
        let patches = diff(Some(&VNode::Fragment(old)), &VNode::Fragment(new));
        // Should remove A (0) and C (2), insert D at adjusted position
        let removes: Vec<_> = patches
            .iter()
            .filter(|p| matches!(p, Patch::RemoveChild { .. }))
            .collect();
        assert_eq!(removes.len(), 2, "should remove A and C");
        let inserts: Vec<_> = patches
            .iter()
            .filter(|p| matches!(p, Patch::InsertChild { .. }))
            .collect();
        assert!(!inserts.is_empty(), "should insert D");
    }

    #[test]
    fn test_diff_fragment_to_element() {
        let old = VNode::Fragment(vec![]);
        let new = VNode::Element(Box::new(VElement::new("div")));
        let patches = diff(Some(&old), &new);
        assert_eq!(patches.len(), 1);
        assert!(matches!(&patches[0], Patch::ReplaceNode { .. }));
    }

    #[test]
    fn test_diff_element_to_fragment() {
        let old = VNode::Element(Box::new(VElement::new("div")));
        let new = VNode::Fragment(vec![VNode::Text(VText::new("a"))]);
        let patches = diff(Some(&old), &new);
        assert_eq!(patches.len(), 1);
        assert!(matches!(&patches[0], Patch::ReplaceNode { .. }));
    }

    #[test]
    fn test_diff_text_to_element() {
        let old = VNode::Text(VText::new("hello"));
        let new = VNode::Element(Box::new(VElement::new("span")));
        let patches = diff(Some(&old), &new);
        assert_eq!(patches.len(), 1);
        assert!(matches!(&patches[0], Patch::ReplaceNode { .. }));
    }

    #[test]
    fn test_diff_identical_trees_produces_no_patches() {
        let tree = VNode::Element(Box::new(
            VElement::new("div")
                .attr("class", "container")
                .child(VNode::Text(VText::new("hello"))),
        ));
        let patches = diff(Some(&tree), &tree);
        assert!(
            patches.is_empty(),
            "identical trees should produce no patches"
        );
    }
}
