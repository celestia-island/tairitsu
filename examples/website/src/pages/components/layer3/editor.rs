use crate::components::breadcrumb;
use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

pub fn render() -> VNode {
    rsx! {
        div { id: "page-component-editor", class: "hikari-page",
            ..vec![breadcrumb(&[("Home", "/"), ("Components", "/components"), ("Layer 3 — Complex", "/components/layer3/editor"), ("Editor", "")])]
            section { class: "page-section",
                h2 { class: "page-section__title", "Editor" }
                p { class: "card__body",
                    "Rich text editor with markdown support, toolbar formatting, code highlighting, and image embedding."
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Markdown Editor" }
                    div { class: "demo-block__body",
                        div { class: "editor-toolbar",
                            button { class: "hi-button hi-button-borderless", "**B**" }
                            button { class: "hi-button hi-button-borderless", "*I*" }
                            button { class: "hi-button hi-button-borderless", "`Code`" }
                            button { class: "hi-button hi-button-borderless", "Link" }
                        }
                        textarea { class: "editor-textarea",
                            placeholder: "Write your content here...",
                            rows: "8"
                        }
                        div { class: "editor-preview",
                            p { "Editor preview area with rendered output." }
                        }
                    }
                }
            }
        }
    }
}
