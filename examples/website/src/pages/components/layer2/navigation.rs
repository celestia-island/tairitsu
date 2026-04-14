use crate::components::breadcrumb;
use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

pub fn render() -> VNode {
    rsx! {
        div { id: "page-component-navigation", class: "hikari-page",
            ..vec![breadcrumb(&[("Home", "/"), ("Components", "/components"), ("Layer 2 — Composed", "/components/layer2/navigation"), ("Navigation", "")])]
            section { class: "page-section",
                h2 { class: "page-section__title", "Navigation" }
                p { class: "card__body",
                    "Navigation components for building menus, breadcrumbs, tabs, and page-level navigation systems."
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Tab Navigation" }
                    div { class: "demo-block__body",
                        div { class: "hi-tabs",
                            div { class: "hi-tab hi-tab-active", "Overview" }
                            div { class: "hi-tab", "API Reference" }
                            div { class: "hi-tab", "Examples" }
                        }
                    }
                }
            }
        }
    }
}
