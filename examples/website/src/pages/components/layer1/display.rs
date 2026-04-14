use crate::components::breadcrumb;
use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

pub fn render() -> VNode {
    rsx! {
        div { id: "page-component-display", class: "hikari-page",
            ..vec![breadcrumb(&[("Home", "/"), ("Components", "/components"), ("Layer 1 — Base", "/components/layer1/display"), ("Display", "")])]
            section { class: "page-section",
                h2 { class: "page-section__title", "Display" }
                p { class: "card__body",
                    "Read-only display components for presenting data: text, numbers, dates, and status indicators."
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Display Variants" }
                    div { class: "demo-block__body",
                        div { class: "display-item",
                            span { class: "display-label", "Status" }
                            span { class: "display-value hi-status-active", "Active" }
                        }
                        div { class: "display-item",
                            span { class: "display-label", "Version" }
                            span { class: "display-value", "0.3.0" }
                        }
                    }
                }
            }
        }
    }
}
