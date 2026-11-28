use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

#[test]
fn test_rsx_simple_element() {
    let node = rsx! {
        div {}
    };

    match node {
        VNode::Element(elem) => {
            assert_eq!(elem.tag, "div");
        }
        _ => panic!("Expected element node"),
    }
}

#[test]
fn test_rsx_element_with_class() {
    let node = rsx! {
        div {
            class: "container"
        }
    };

    match node {
        VNode::Element(elem) => {
            assert_eq!(elem.tag, "div");
            assert_eq!(elem.class.to_string(), "container");
        }
        _ => panic!("Expected element node"),
    }
}

#[test]
fn test_rsx_element_with_child() {
    let node = rsx! {
        div {
            "Hello"
        }
    };

    match node {
        VNode::Element(elem) => {
            assert_eq!(elem.tag, "div");
            assert_eq!(elem.children.len(), 1);
        }
        _ => panic!("Expected element node"),
    }
}

#[test]
fn test_rsx_nested_elements() {
    let node = rsx! {
        div {
            span {}
        }
    };

    match node {
        VNode::Element(elem) => {
            assert_eq!(elem.tag, "div");
            assert_eq!(elem.children.len(), 1);
        }
        _ => panic!("Expected parent element"),
    }
}

#[test]
fn test_rsx_with_style() {
    let node = rsx! {
        div {
            style: "color: red"
        }
    };

    match node {
        VNode::Element(elem) => {
            assert!(!elem.style.static_styles.is_empty());
        }
        _ => panic!("Expected styled element"),
    }
}

/// An `if` without an `else` renders nothing when it is false: no extra empty
/// text node, so the siblings keep their indices (Dioxus's rsx does the same).
/// MUT:rsx-if-no-else-empty — with the arm back on `VNode::empty()` this test
/// reds with `left: 3`, `right: 2`.
#[test]
fn test_rsx_if_without_else_renders_nothing_when_false() {
    let show = false;
    let node = rsx! {
        div {
            "before"
            if show {
                span { class: "optional" }
            }
            "after"
        }
    };

    match &node {
        VNode::Element(elem) => {
            assert_eq!(elem.children.len(), 2);
        }
        _ => panic!("Expected element node"),
    }
    assert_eq!(node.render_to_html(), "<div>beforeafter</div>");
}

/// The `true` branch keeps rendering its children, in order.
/// MUT:rsx-if-no-else-empty — this test stays green under that mutation: it
/// pins the branch the change does not touch.
#[test]
fn test_rsx_if_without_else_renders_its_branch_when_true() {
    let show = true;
    let node = rsx! {
        div {
            "before"
            if show {
                span { class: "optional" }
                span { class: "second" }
            }
            "after"
        }
    };

    match &node {
        VNode::Element(elem) => {
            assert_eq!(elem.children.len(), 4);
        }
        _ => panic!("Expected element node"),
    }
    assert_eq!(
        node.render_to_html(),
        r#"<div>before<span class="optional"></span><span class="second"></span>after</div>"#
    );
}

/// A lone `if` without an `else` at the root is an empty fragment when it is
/// false: it renders zero nodes, never an empty text node.
/// MUT:rsx-if-no-else-empty — with the arm back on `VNode::empty()` this test
/// reds at the `matches!`.
#[test]
fn test_rsx_root_if_without_else_is_an_empty_fragment_when_false() {
    let node = rsx! {
        if false {
            div { "never" }
        }
    };

    assert!(matches!(node, VNode::Fragment(ref children) if children.is_empty()));
    assert_eq!(node.render_to_html(), "");
}
