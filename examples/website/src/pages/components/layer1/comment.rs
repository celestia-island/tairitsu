use crate::components::breadcrumb;
use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

pub fn render() -> VNode {
    rsx! {
        div { id: "page-component-comment", class: "hikari-page",
            ..vec![breadcrumb(&[("Home", "/"), ("Components", "/components"), ("Layer 1 — Base", "/components/layer1/comment"), ("Comment", "")])]
            section { class: "page-section",
                h2 { class: "page-section__title", "Comment" }
                p { class: "card__body",
                    "Comment/discussion component with nested replies, author info, timestamp, and actions."
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Basic Comment" }
                    div { class: "demo-block__body",
                        div { class: "hi-comment",
                            div { class: "hi-comment-author", "Alice" }
                            div { class: "hi-comment-content", "This is a sample comment demonstrating the comment component." }
                            div { class: "hi-comment-meta", "2024-01-15 10:30" }
                        }
                    }
                }
            }
        }
    }
}
