use crate::components::breadcrumb;
use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

pub fn render() -> VNode {
    rsx! {
        div { id: "page-component-avatar", class: "hikari-page",
            ..vec![breadcrumb(&[("Home", "/"), ("Components", "/components"), ("Layer 1 — Base", "/components/layer1/avatar"), ("Avatar", "")])]
            section { class: "page-section",
                h2 { class: "page-section__title", "Avatar" }
                p { class: "card__body",
                    "User avatar component with image fallback, size variants, and group display support."
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Avatar Sizes" }
                    div { class: "demo-block__body",
                        div { class: "avatar-row",
                            div { class: "hi-avatar hi-avatar-sm", "A" }
                            div { class: "hi-avatar hi-avatar-md", "B" }
                            div { class: "hi-avatar hi-avatar-lg", "C" }
                            div { class: "hi-avatar hi-avatar-xl", "D" }
                        }
                    }
                }
            }
        }
    }
}
