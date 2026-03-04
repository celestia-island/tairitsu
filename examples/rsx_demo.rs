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
}
