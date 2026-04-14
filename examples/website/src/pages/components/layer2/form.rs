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
    let grid = VNode::Element(
        el("div")
            .attr(
                "style",
                "display:grid;grid-template-columns:1fr 1fr;gap:16px;",
            )
            .children(vec![
                VNode::Element(el("div").class("form-group").children(vec![
            VNode::Element(el("label").child(txt("First Name"))),
            VNode::Element(el("input").attr("type","text").attr("placeholder","First name")),
        ])),
                VNode::Element(el("div").class("form-group").children(vec![
            VNode::Element(el("label").child(txt("Last Name"))),
            VNode::Element(el("input").attr("type","text").attr("placeholder","Last name")),
        ])),
                VNode::Element(el("div").class("form-group").children(vec![
            VNode::Element(el("label").child(txt("Email"))),
            VNode::Element(el("input").attr("type","email").attr("placeholder","Email address")),
        ])),
                VNode::Element(el("div").class("form-group").children(vec![
            VNode::Element(el("label").child(txt("Phone"))),
            VNode::Element(el("input").attr("type","tel").attr("placeholder","Phone number")),
        ])),
            ]),
    );
    rsx! {
        div { id: "page-component-form-2", class: "hikari-page",
            ..vec![breadcrumb(&[("Home", "/"), ("Components", "/components"), ("Layer 2 — Composed", "/components/layer2/form"), ("Form", "")])]
            section { class: "page-section",
                h2 { class: "page-section__title", "Form (Composed)" }
                p { class: "card__body",
                    "Advanced form patterns: multi-step forms, form layouts with grid, dynamic field arrays, and complex validation."
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Multi-column Form Layout" }
                    div { class: "demo-block__body",
                        ..vec![grid]
                    }
                }
            }
        }
    }
}
