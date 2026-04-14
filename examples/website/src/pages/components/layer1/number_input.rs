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
    let num_input = VNode::Element(
        el("input")
            .attr("type", "number")
            .attr("value", "0")
            .attr("min", "0")
            .attr("max", "100")
            .attr("step", "1"),
    );
    rsx! {
        div { id: "page-component-number-input", class: "hikari-page",
            ..vec![breadcrumb(&[("Home", "/"), ("Components", "/components"), ("Layer 1 — Base", "/components/layer1/number-input"), ("Number Input", "")])]
            section { class: "page-section",
                h2 { class: "page-section__title", "Number Input" }
                p { class: "card__body",
                    "Input field for numeric values with increment/decrement controls, step configuration, and range validation."
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Basic Number Input" }
                    div { class: "demo-block__body",
                        div { class: "form-group",
                            label { "Quantity" }
                            ..vec![num_input]
                        }
                    }
                }
            }
        }
    }
}
