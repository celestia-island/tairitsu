use crate::components::breadcrumb;
use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

pub fn render() -> VNode {
    rsx! {
        div { id: "page-component-button", class: "hikari-page",
            ..vec![breadcrumb(&[("Home", "/"), ("Components", "/components"), ("Layer 1 — Base", "/components/layer1/button"), ("Button", "")])]
            section { class: "page-section",
                h2 { class: "page-section__title", "Button" }
                p { class: "card__body",
                    "A fundamental interactive element that triggers an action when clicked. Supports multiple variants, sizes, and states."
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Basic Usage" }
                    div { class: "demo-block__body",
                        a { href: "#", class: "hi-button hi-button-primary", "Primary Button" }
                        " "
                        a { href: "#", class: "hi-button hi-button-secondary", "Secondary Button" }
                        " "
                        a { href: "#", class: "hi-button hi-button-tertiary", "Tertiary Button" }
                    }
                }
            }
        }
    }
}
