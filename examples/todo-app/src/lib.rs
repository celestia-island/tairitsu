use tairitsu_macros::rsx;
use tairitsu_vdom::{Signal, VNode};

fn todo_app() -> VNode {
    rsx! {
        div {
            class: "todo-app",
            style: "max-width: 400px; margin: 40px auto; font-family: sans-serif",

            h1 {
                "Todo App"
            }

            ul {
                class: "todo-list",

                li {
                    class: "todo-item",
                    "Learn Tairitsu"
                }

                li {
                    class: "todo-item",
                    "Build an app"
                }

                li {
                    class: "todo-item",
                    "Ship to production"
                }
            }

            div {
                class: "todo-input",
                style: "margin-top: 16px",

                input {
                    placeholder: "What needs to be done?",
                }

                button {
                    class: "add-btn",
                    style: "margin-left: 8px",
                    "Add"
                }
            }
        }
    }
}

#[allow(dead_code)]
fn dynamic_counter() -> VNode {
    let _count: Signal<i32> = Signal::new(0);

    rsx! {
        div {
            class: "counter",

            h2 {
                "Reactive Counter"
            }

            button {
                class: "increment-btn",
                "Increment"
            }
        }
    }
}

pub fn main() {
    let vnode = todo_app();
    let html = vnode.render_to_html();
    println!("{}", html);
}

#[cfg(test)]
mod tests {
    use super::*;
    use tairitsu_vdom::IntoVNodeChild;

    #[test]
    fn test_todo_app_renders() {
        let node = todo_app();
        match node {
            VNode::Element(elem) => {
                assert_eq!(elem.tag, "div");
                assert!(elem.class.static_classes.contains("todo-app"));
            }
            _ => panic!("Expected element"),
        }
    }

    #[test]
    fn test_todo_app_has_list() {
        let node = todo_app();
        match node {
            VNode::Element(elem) => {
                let list = elem
                    .children
                    .iter()
                    .find(|c| matches!(c, VNode::Element(e) if e.tag == "ul"));
                assert!(list.is_some(), "Should contain a <ul>");
            }
            _ => panic!("Expected element"),
        }
    }

    #[test]
    fn test_todo_app_has_input() {
        let node = todo_app();
        match node {
            VNode::Element(elem) => {
                let has_input = elem
                    .children
                    .iter()
                    .any(|c| matches!(c, VNode::Element(e) if e.tag == "div"));
                assert!(has_input);
            }
            _ => panic!("Expected element"),
        }
    }

    #[test]
    fn test_dynamic_counter() {
        let node = dynamic_counter();
        match node {
            VNode::Element(elem) => {
                assert_eq!(elem.tag, "div");
                assert!(elem.class.static_classes.contains("counter"));
            }
            _ => panic!("Expected element"),
        }
    }

    #[test]
    fn test_signal_into_vnode_child() {
        let count: Signal<i32> = Signal::new(42);
        let node = count.into_vnode_child();
        match node {
            VNode::DynamicText(dt) => assert_eq!(dt.initial, "42"),
            _ => panic!("Expected DynamicText"),
        }
    }

    #[test]
    fn test_signal_string_into_vnode_child() {
        let name: Signal<String> = Signal::new("Tairitsu".to_string());
        let node = name.into_vnode_child();
        match node {
            VNode::DynamicText(dt) => assert_eq!(dt.initial, "Tairitsu"),
            _ => panic!("Expected DynamicText"),
        }
    }

    #[test]
    fn test_todo_app_render_html() {
        let html = todo_app().render_to_html();
        assert!(html.contains("todo-app"));
        assert!(html.contains("Todo App"));
        assert!(html.contains("Learn Tairitsu"));
    }
}
