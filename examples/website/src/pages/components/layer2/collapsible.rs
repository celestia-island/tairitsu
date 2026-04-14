use crate::components::breadcrumb;
use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

pub fn render() -> VNode {
    rsx! {
        div { id: "page-component-collapsible", class: "hikari-page",
            ..vec![breadcrumb(&[("Home", "/"), ("Components", "/components"), ("Layer 2 — Composed", "/components/layer2/collapsible"), ("Collapsible", "")])]
            section { class: "page-section",
                h2 { class: "page-section__title", "Collapsible" }
                p { class: "card__body",
                    "Expandable/collapsible content panel with smooth animation and custom trigger."
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Basic Collapsible" }
                    div { class: "demo-block__body",
                        details { class: "hi-collapsible",
                            summary { "Click to expand" }
                            p { "This content is hidden by default and shown when the user clicks the summary." }
                        }
                    }
                }
            }
        }
    }
}
