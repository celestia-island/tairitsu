use crate::components::breadcrumb;
use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

pub fn render() -> VNode {
    rsx! {
        div { id: "page-component-tag", class: "hikari-page",
            ..vec![breadcrumb(&[("Home", "/"), ("Components", "/components"), ("Layer 1 — Base", "/components/layer1/tag"), ("Tag", "")])]
            section { class: "page-section",
                h2 { class: "page-section__title", "Tag" }
                p { class: "card__body",
                    "Labeling and categorisation component with color variants, closable option, and icon support."
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Tag Variants" }
                    div { class: "demo-block__body",
                        span { class: "hi-tag hi-tag-primary", "Primary" }
                        " "
                        span { class: "hi-tag hi-tag-success", "Success" }
                        " "
                        span { class: "hi-tag hi-tag-warning", "Warning" }
                        " "
                        span { class: "hi-tag hi-tag-error", "Error" }
                        " "
                        span { class: "hi-tag hi-tag-default", "Default" }
                    }
                }
            }
        }
    }
}
