use crate::components::breadcrumb;
use tairitsu_macros::rsx;
use tairitsu_vdom::{VElement, VNode, VText};

fn txt(s: &str) -> VNode {
    VNode::Text(VText::new(s))
}
fn el(tag: &str) -> VElement {
    VElement::new(tag)
}

pub fn render() -> VNode {
    let username_input = VNode::Element(
        el("input")
            .attr("type", "text")
            .attr("placeholder", "Enter username"),
    );
    let password_input = VNode::Element(
        el("input")
            .attr("type", "password")
            .attr("placeholder", "Enter password"),
    );
    rsx! {
        div { id: "page-component-form", class: "hikari-page",
            ..vec![breadcrumb(&[("Home", "/"), ("Components", "/components"), ("Layer 1 — Base", "/components/layer1/form"), ("Form", "")])]
            section { class: "page-section",
                h2 { class: "page-section__title", "Form" }
                p { class: "card__body",
                    "Form container for collecting and validating user input. Provides layout structure and validation integration."
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Basic Form Layout" }
                    div { class: "demo-block__body",
                        div { class: "form-group",
                            label { "Username" }
                            div { class: "form-input-wrapper", ..vec![username_input] }
                        }
                        div { class: "form-group",
                            label { "Password" }
                            div { class: "form-input-wrapper", ..vec![password_input] }
                        }
                    }
                }
            }
        }
    }
}
