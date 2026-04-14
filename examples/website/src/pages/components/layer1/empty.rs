use crate::components::breadcrumb;
use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

pub fn render() -> VNode {
    rsx! {
        div { id: "page-component-empty", class: "hikari-page",
            ..vec![breadcrumb(&[("Home", "/"), ("Components", "/components"), ("Layer 1 — Base", "/components/layer1/empty"), ("Empty", "")])]
            section { class: "page-section",
                h2 { class: "page-section__title", "Empty" }
                p { class: "card__body",
                    "Placeholder state for empty data lists with customisable description and action button."
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Basic Empty State" }
                    div { class: "demo-block__body",
                        div { class: "hi-empty",
                            div { class: "hi-empty-icon", "\u{1F4E6}" }
                            p { class: "hi-empty-description", "No data available" }
                            a { href: "#", class: "hi-button hi-button-primary", "Create Now" }
                        }
                    }
                }
            }
        }
    }
}
