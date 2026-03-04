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
