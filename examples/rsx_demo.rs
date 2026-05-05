use tairitsu_hooks::{use_effect, use_signal, use_state};
use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

/// Example component using the rsx! macro
pub fn counter_component() -> VNode {
    let (count, set_count) = use_state(0);
    let signal = use_signal(0);

    use_effect(move || {
        // This effect runs when signal changes
        let _ = signal.get();
    });

    rsx! {
        div {
            class: "counter-container",
            style: "padding: 20px",

            h1 {
                class: "counter-title",
                "Counter Example"
            }

            p {
                class: "counter-value",
                "Count: "
            }

            button {
                class: "counter-button",
                "Increment"
            }
        }
    }
}

/// Example: Simple card component
pub fn card_component(title: &str, content: &str) -> VNode {
    rsx! {
        div {
            class: "card",
            style: "border: 1px solid #ccc; padding: 16px; margin: 8px",

            h2 {
                class: "card-title",
            }

            p {
                class: "card-content",
            }
        }
    }
}

/// Example: List component with children
pub fn list_component() -> VNode {
    rsx! {
        ul {
            class: "item-list",

            li {
                class: "list-item",
                "First item"
            }

            li {
                class: "list-item",
                "Second item"
            }

            li {
                class: "list-item",
                "Third item"
            }
        }
    }
}

fn make_text_node(s: &str) -> VNode {
    VNode::Text(tairitsu_vdom::VText::new(s))
}

pub fn single_brace_root() -> VNode {
    rsx! {
        {make_text_node("hello")}
    }
}

pub fn mixed_roots_with_brace() -> VNode {
    rsx! {
        div { "first" }
        {make_text_node("second")}
        span { "third" }
    }
}

pub fn mixed_brace_and_control_flow() -> VNode {
    let show = true;
    rsx! {
        {make_text_node("dynamic")}
        if show {
            span { "conditional" }
        }
    }
}

pub fn multiple_brace_roots() -> VNode {
    rsx! {
        {make_text_node("a")}
        {make_text_node("b")}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_counter_component() {
        let node = counter_component();

        match node {
            VNode::Element(elem) => {
                assert_eq!(elem.tag, "div");
                assert!(elem.class.to_string().contains("counter"));
            }
            _ => panic!("Expected element"),
        }
    }

    #[test]
    fn test_card_component() {
        let node = card_component("Test", "Content");

        match node {
            VNode::Element(elem) => {
                assert_eq!(elem.tag, "div");
                assert_eq!(elem.children.len(), 2);
            }
            _ => panic!("Expected card element"),
        }
    }

    #[test]
    fn test_list_component() {
        let node = list_component();

        match node {
            VNode::Element(elem) => {
                assert_eq!(elem.tag, "ul");
                assert_eq!(elem.children.len(), 3);
            }
            _ => panic!("Expected list element"),
        }
    }

    #[test]
    fn test_single_brace_root() {
        let node = single_brace_root();
        match node {
            VNode::Fragment(children) => {
                assert_eq!(children.len(), 1);
                match &children[0] {
                    VNode::Text(t) => assert_eq!(t.text, "hello"),
                    _ => panic!("Expected text node"),
                }
            }
            _ => panic!("Expected fragment, got {:?}", node),
        }
    }

    #[test]
    fn test_mixed_roots_with_brace() {
        let node = mixed_roots_with_brace();
        match node {
            VNode::Fragment(children) => {
                assert_eq!(children.len(), 3);
                match &children[0] {
                    VNode::Element(e) => assert_eq!(e.tag, "div"),
                    _ => panic!("Expected element"),
                }
                match &children[2] {
                    VNode::Element(e) => assert_eq!(e.tag, "span"),
                    _ => panic!("Expected element"),
                }
            }
            _ => panic!("Expected fragment"),
        }
    }

    #[test]
    fn test_mixed_brace_and_control_flow() {
        let node = mixed_brace_and_control_flow();
        match node {
            VNode::Fragment(children) => {
                assert_eq!(children.len(), 2);
            }
            _ => panic!("Expected fragment"),
        }
    }

    #[test]
    fn test_multiple_brace_roots() {
        let node = multiple_brace_roots();
        match node {
            VNode::Fragment(children) => {
                assert_eq!(children.len(), 2);
            }
            _ => panic!("Expected fragment"),
        }
    }
}
