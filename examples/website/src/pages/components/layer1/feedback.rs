use crate::components::breadcrumb;
use tairitsu_macros::rsx;
use tairitsu_vdom::{VElement, VNode, VText};

fn el(tag: &str) -> VElement {
    VElement::new(tag)
}
fn txt(s: &str) -> VNode {
    VNode::Text(VText::new(s))
}

pub fn render() -> VNode {
    rsx! {
        div { id: "page-component-feedback", class: "hikari-page",
            ..vec![breadcrumb(&[("Home", "/"), ("Components", "/components"), ("Layer 1 \u{2014} Base", "/components/layer1/feedback"), ("Feedback", "")])]
            section { class: "page-section",
                h2 { class: "page-section__title", "Feedback" }
                p { class: "page-section__description",
                    "Visual feedback components: alerts, messages, notifications, and loading indicators."
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Alert Variants" }
                    div { class: "demo-block__body",
                        div { class: "hi-alert hi-alert-info", "Info: This is an informational message." }
                        div { class: "hi-alert hi-alert-success", "Success: Operation completed successfully." }
                        div { class: "hi-alert hi-alert-warning", "Warning: Please review before proceeding." }
                        div { class: "hi-alert hi-alert-error", "Error: Something went wrong." }
                    }
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Alert with Title" }
                    div { class: "demo-block__body",
                        div { class: "hi-alert hi-alert-info",
                            div { style: "display:flex;align-items:baseline;gap:8px;margin-bottom:4px;",
                                strong { "Information" }
                            }
                            div { "Your account settings have been updated." }
                        }
                        div { class: "hi-alert hi-alert-warning",
                            div { style: "display:flex;align-items:baseline;gap:8px;margin-bottom:4px;",
                                strong { "Deprecated" }
                            }
                            div { "This API will be removed in version 2.0." }
                        }
                    }
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Inline Status Badges" }
                    div { class: "demo-block__body demo-block__body--horizontal",
                        span { class: "hi-badge hi-badge-success", "Online" }
                        span { class: "hi-badge hi-badge-warning", "Pending" }
                        span { class: "hi-badge hi-badge-error", "Offline" }
                        span { class: "hi-badge hi-badge-primary", "Active" }
                        span { class: "hi-badge hi-badge-default", "Inactive" }
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
                                tr { td { code { "variant" } } td { code { "info | success | warning | error" } } td { "Alert type" } }
                                tr { td { code { "title" } } td { code { "string" } } td { "Optional alert title" } }
                                tr { td { code { "closable" } } td { code { "bool" } } td { "Show close button" } }
                                tr { td { code { "onClose" } } td { code { "() => void" } } td { "Close callback" } }
                            }
                        }
                    }
                }
            }
        }
    }
}
