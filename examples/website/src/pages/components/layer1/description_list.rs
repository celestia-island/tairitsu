use crate::components::breadcrumb;
use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

pub fn render() -> VNode {
    rsx! {
        div { id: "page-component-description-list", class: "hikari-page",
            ..vec![breadcrumb(&[("Home", "/"), ("Components", "/components"), ("Layer 1 — Base", "/components/layer1/description-list"), ("Description List", "")])]
            section { class: "page-section",
                h2 { class: "page-section__title", "Description List" }
                p { class: "card__body",
                    "Key-value pair display component for presenting metadata and configuration details."
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Basic Description List" }
                    div { class: "demo-block__body",
                        dl { class: "hi-description-list",
                            dt { class: "hi-dl-term", "Name" }
                            dd { class: "hi-dl-detail", "Tairitsu" }
                            dt { class: "hi-dl-term", "Version" }
                            dd { class: "hi-dl-detail", "0.3.0" }
                            dt { class: "hi-dl-term", "Runtime" }
                            dd { class: "hi-dl-detail", "wasm32-wasip2" }
                        }
                    }
                }
            }
        }
    }
}
