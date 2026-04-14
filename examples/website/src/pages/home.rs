//! Home page — mirrors hikari-legacy home page structure (dark theme variant).
//!
//! Centered hero with gradient text title, subtitle, description,
//! and two CTA buttons — matching hikari's Layout > Container > Section > Row structure.

use tairitsu_macros::rsx;
use tairitsu_vdom::{VElement, VNode, VText};

use crate::components::svg_icon;
use hikari_icons::MdiIcon;

fn txt(s: &str) -> VNode {
    VNode::Text(VText::new(s))
}
fn el(tag: &str) -> VElement {
    VElement::new(tag)
}

pub fn render() -> VNode {
    rsx! {
        div { id: "page-home", class: "hikari-page is-active",
            section { class: "hero-section",
                div { class: "hero-content",
                    div { class: "hero-logo",
                        ..vec![svg_icon(MdiIcon::CubeOutline, 64, "hero-logo-icon")]
                    }
                    h1 { class: "hero-title", "Tairitsu" }
                    p { class: "hero-subtitle",
                        "A comprehensive WASM Component Runtime Engine."
                    }
                    p { class: "hero-description",
                        "Built with a reactive virtual DOM, compiled to WebAssembly.",
                        " Tairitsu provides a rich set of components from basic primitives",
                        " to complex data visualisations \u{2014} all rendered without JavaScript."
                    }
                    p { class: "hero-tagline",
                        "Docker-like architecture for WASM modules. Type-safe via WIT."
                    }
                }
                div { class: "hero-actions",
                    a {
                        href: "/components/layer1/button",
                        class: "hi-button hi-button-primary hi-button-lg hero-btn",
                        "Explore Components"
                    }
                    a {
                        href: "/guides/quick-start",
                        class: "hi-button hi-button-secondary hi-button-lg hero-btn",
                        "Quick Start"
                    }
                }
            }
            div { class: "hero-spacer" }
            section { class: "features-section",
                h2 { class: "features-title", "What is Tairitsu?" }
                div { class: "features-grid",
                    ..vec![
                        feature_card(
                            "Component Library",
                            "Layered architecture: Layer 1 (base primitives), Layer 2 (composed patterns), Layer 3 (complex widgets).",
                            MdiIcon::Package,
                        ),
                        feature_card(
                            "Design System",
                            "500+ traditional Chinese colours, CSS utility classes, icon library, animations, and i18n system.",
                            MdiIcon::Palette,
                        ),
                        feature_card(
                            "WebAssembly First",
                            "Ships as a wasm32-wasip2 component. Rendered with the Tairitsu virtual DOM \u{2014} no JavaScript framework required.",
                            MdiIcon::CubeOutline,
                        ),
                    ]
                }
            }
        }
    }
}

fn feature_card(title: &str, desc: &str, icon: MdiIcon) -> VNode {
    VNode::Element(el("div").class("feature-card").children(vec![
        VNode::Element(el("div").class("feature-icon-wrap").child(svg_icon(
            icon,
            36,
            "feature-icon",
        ))),
        VNode::Element(el("h3").class("feature-card-title").child(txt(title))),
        VNode::Element(el("p").class("feature-card-desc").child(txt(desc))),
    ]))
}
