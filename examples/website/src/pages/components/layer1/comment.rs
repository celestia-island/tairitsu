use crate::components::breadcrumb;
use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

pub fn render() -> VNode {
    rsx! {
        div { id: "page-component-comment", class: "hikari-page",
            ..vec![breadcrumb(&[("Home", "/"), ("Components", "/components"), ("Layer 1 \u{2014} Base", "/components/layer1/comment"), ("Comment", "")])]
            section { class: "page-section",
                h2 { class: "page-section__title", "Comment" }
                p { class: "card__body",
                    "Comment/discussion component with nested replies, author info, timestamp, and actions."
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Basic Comment" }
                    div { class: "demo-block__body",
                        div { class: "hi-comment",
                            div { class: "hi-comment-header",
                                div { style: "display:flex;align-items:center;gap:8px;",
                                    div { class: "hi-avatar hi-avatar-sm", "A" }
                                    span { class: "hi-comment-author", "Alice" }
                                }
                                span { style: "font-size:0.75rem;color:var(--hi-color-text-disabled);", "2 hours ago" }
                            }
                            div { class: "hi-comment-content", "This is a sample comment demonstrating the comment component." }
                            div { class: "hi-comment-actions",
                                a { href: "#", "Reply" }
                                a { href: "#", "Like" }
                            }
                        }
                    }
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Comment Thread" }
                    div { class: "demo-block__body",
                        div { style: "display:flex;flex-direction:column;gap:12px;",
                            div { class: "hi-comment",
                                div { class: "hi-comment-header",
                                    div { style: "display:flex;align-items:center;gap:8px;",
                                        div { class: "hi-avatar hi-avatar-sm", style: "background:#e91e63;", "B" }
                                        span { class: "hi-comment-author", "Bob" }
                                    }
                                    span { style: "font-size:0.75rem;color:var(--hi-color-text-disabled);", "3 hours ago" }
                                }
                                div { class: "hi-comment-content", "The new component architecture looks great! I especially like the layer separation." }
                                div { class: "hi-comment-actions",
                                    a { href: "#", "Reply" }
                                    a { href: "#", "Like (3)" }
                                }
                            }
                            div { class: "hi-comment", style: "margin-left:32px;" ,
                                div { class: "hi-comment-header",
                                    div { style: "display:flex;align-items:center;gap:8px;",
                                        div { class: "hi-avatar hi-avatar-sm", "A" }
                                        span { class: "hi-comment-author", "Alice" }
                                    }
                                    span { style: "font-size:0.75rem;color:var(--hi-color-text-disabled);", "2 hours ago" }
                                }
                                div { class: "hi-comment-content", "Thanks! The three-layer approach makes it easy to compose complex UIs from simple primitives." }
                                div { class: "hi-comment-actions",
                                    a { href: "#", "Reply" }
                                    a { href: "#", "Like (1)" }
                                }
                            }
                            div { class: "hi-comment",
                                div { class: "hi-comment-header",
                                    div { style: "display:flex;align-items:center;gap:8px;",
                                        div { class: "hi-avatar hi-avatar-sm", style: "background:#2196f3;", "C" }
                                        span { class: "hi-comment-author", "Charlie" }
                                    }
                                    span { style: "font-size:0.75rem;color:var(--hi-color-text-disabled);", "1 hour ago" }
                                }
                                div { class: "hi-comment-content", "Would love to see more documentation examples. The current ones are helpful but more edge cases would be nice." }
                                div { class: "hi-comment-actions",
                                    a { href: "#", "Reply" }
                                    a { href: "#", "Like" }
                                }
                            }
                        }
                    }
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Comment with Input" }
                    div { class: "demo-block__body",
                        div { style: "display:flex;gap:12px;align-items:flex-start;",
                            div { class: "hi-avatar hi-avatar-sm", style: "margin-top:2px;", "Y" }
                            div { style: "flex:1;",
                                div { class: "form-input-wrapper",
                                    ..vec![VNode::Element(
                                        tairitsu_vdom::VElement::new("input").attr("type","text").attr("placeholder","Write a comment...")
                                    )]
                                }
                                div { style: "display:flex;justify-content:flex-end;margin-top:8px;",
                                    a { href: "#", class: "hi-button hi-button-primary", style: "padding:4px 16px;font-size:0.8125rem;", "Post" }
                                }
                            }
                        }
                    }
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "API" }
                    div { class: "demo-block__body",
                        table { class: "api-table",
                            thead {
                                tr { th { "Property" } th { "Type" } th { "Description" } }
                            }
                            tbody {
                                tr { td { code { "author" } } td { code { "string" } } td { "Author name" } }
                                tr { td { code { "avatar" } } td { code { "string" } } td { "Author avatar URL or initial" } }
                                tr { td { code { "content" } } td { code { "string" } } td { "Comment body text" } }
                                tr { td { code { "datetime" } } td { code { "string" } } td { "Timestamp" } }
                                tr { td { code { "actions" } } td { code { "string[]" } } td { "Action labels (reply, like, etc.)" } }
                                tr { td { code { "replies" } } td { code { "Comment[]" } } td { "Nested reply comments" } }
                            }
                        }
                    }
                }
            }
        }
    }
}
