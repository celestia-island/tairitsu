//! Home page — mirrors hikari-legacy home page structure (dark theme variant).
//!
//! Centered hero with gradient background, logo, title, subtitle, CTA buttons,
//! feature cards, and architecture layers — matching hikari's layout exactly.

use tairitsu_macros::rsx;
use tairitsu_vdom::{VElement, VNode, VText};

use crate::components::breadcrumb;

fn txt(s: &str) -> VNode {
    VNode::Text(VText::new(s))
}
fn el(tag: &str) -> VElement {
    VElement::new(tag)
}

pub fn render() -> VNode {
    rsx! {
        div { id: "page-home", class: "hikari-page is-active",
            ..vec![breadcrumb(&[("Home", "")])]
            section { class: "page-hero",
                div { class: "page-hero__inner",
                    h1 { class: "page-hero__title", "Tairitsu" }
                    p { class: "page-hero__subtitle",
                        "A generic WASM Component Runtime Engine."
                    }
                    p { class: "page-hero__desc",
                        "Docker-like architecture for WASM modules. Type-safe via WIT."
                    }
                    div { class: "page-hero__actions",
                        a {
                            href: "/guides/quick-start",
                            class: "hi-button hi-button-primary hi-button-lg",
                            "Quick Start \u{2192}"
                        }
                        a {
                            href: "/system/overview",
                            class: "hi-button hi-button-secondary hi-button-lg",
                            "System Architecture"
                        }
                    }
                }
            }
            section { class: "page-section",
                h2 { class: "page-section__title", "What is Tairitsu?" }
                div { class: "card-grid",
                    ..vec![
                        VNode::Element(el("div").class("card").children(vec![
                            VNode::Element(el("h3").class("card__title").child(txt("Image / Container Model"))),
                            VNode::Element(el("p").class("card__body").child(txt("Docker-like architecture for managing WASM modules."))),
                        ])),
                        VNode::Element(el("div").class("card").children(vec![
                            VNode::Element(el("h3").class("card__title").child(txt("Generic Runtime"))),
                            VNode::Element(el("p").class("card__body").child(txt("No preset WIT interfaces. Pluggable host imports and guest exports."))),
                        ])),
                        VNode::Element(el("div").class("card").children(vec![
                            VNode::Element(el("h3").class("card__title").child(txt("Builder Pattern"))),
                            VNode::Element(el("p").class("card__body").child(txt("Flexible Container::builder() API for configuring host imports."))),
                        ])),
                    ]
                }
            }
            section { class: "page-section",
                h2 { class: "page-section__title", "Architecture Layers" }
                div { class: "card-grid",
                    ..vec![
                        VNode::Element(el("div").class("card").children(vec![
                            VNode::Element(el("h3").class("card__title").child(txt("App Layer"))),
                            VNode::Element(el("p").class("card__body").child(txt("Custom WIT interfaces, business components, example applications."))),
                        ])),
                        VNode::Element(el("div").class("card").children(vec![
                            VNode::Element(el("h3").class("card__title").child(txt("Framework Layer"))),
                            VNode::Element(el("p").class("card__body").child(txt("Runtime + macros + vdom/hooks/style/web + packager."))),
                        ])),
                        VNode::Element(el("div").class("card").children(vec![
                            VNode::Element(el("h3").class("card__title").child(txt("Host Layer"))),
                            VNode::Element(el("p").class("card__body").child(txt("wasmtime/native host and browser-glue runtime adaptors."))),
                        ])),
                    ]
                }
            }
        }
    }
}
