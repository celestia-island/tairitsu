//! Component documentation pages — Layer 1 (base), Layer 2 (composed), Layer 3 (complex).

pub mod layer1;
pub mod layer2;
pub mod layer3;

use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

use crate::components::breadcrumb;

pub fn render_all() -> Vec<VNode> {
    let mut pages = Vec::new();
    pages.push(render_overview());
    pages.extend(layer1::render_all());
    pages.extend(layer2::render_all());
    pages.extend(layer3::render_all());
    pages
}

pub fn render_overview() -> VNode {
    rsx! {
        div { id: "page-components", class: "hikari-page",
            ..vec![breadcrumb(&[("Home", "/"), ("Components", "")])]
            section { class: "page-section",
                h2 { class: "page-section__title", "Components" }
                p { class: "card__body",
                    "Tairitsu provides a rich set of UI components organised into three layers of complexity."
                }
                div { class: "card-grid",
                    div { class: "card",
                        h3 { class: "card__title", "Layer 1 — Base" }
                        p { class: "card__body",
                            "Fundamental building blocks: Button, Form, Input, Switch, and more."
                        }
                        a { href: "/components/layer1/button", class: "hi-button hi-button-secondary", "View Components" }
                    }
                    div { class: "card",
                        h3 { class: "card__title", "Layer 2 — Composed" }
                        p { class: "card__body",
                            "Higher-level patterns built from base primitives: Navigation, Table, Tree, etc."
                        }
                        a { href: "/components/layer2/navigation", class: "hi-button hi-button-secondary", "View Components" }
                    }
                    div { class: "card",
                        h3 { class: "card__title", "Layer 3 — Complex" }
                        p { class: "card__body",
                            "Full-featured widgets: Media player, Editor, Visualization, User Guide."
                        }
                        a { href: "/components/layer3/media", class: "hi-button hi-button-secondary", "View Components" }
                    }
                }
            }
        }
    }
}
