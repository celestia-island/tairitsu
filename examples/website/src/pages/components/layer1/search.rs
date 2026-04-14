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
    let search_input = VNode::Element(
        el("input")
            .attr("type", "text")
            .attr("placeholder", "Search..."),
    );
    rsx! {
        div { id: "page-component-search", class: "hikari-page",
            ..vec![breadcrumb(&[("Home", "/"), ("Components", "/components"), ("Layer 1 — Base", "/components/layer1/search"), ("Search", "")])]
            section { class: "page-section",
                h2 { class: "page-section__title", "Search" }
                p { class: "card__body",
                    "Search input with optional icon, clear button, and debounced search callback support."
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Basic Search" }
                    div { class: "demo-block__body",
                        div { class: "search-wrapper",
                            span { class: "search-icon", "\u{1F50D}" }
                            ..vec![search_input]
                        }
                    }
                }
            }
        }
    }
}
