use crate::components::breadcrumb;
use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

pub fn render() -> VNode {
    rsx! {
        div { id: "page-component-feedback", class: "hikari-page",
            ..vec![breadcrumb(&[("Home", "/"), ("Components", "/components"), ("Layer 1 \u{2014} Base", "/components/layer1/feedback"), ("Feedback", "")])]
            section { class: "page-section",
                h2 { class: "page-section__title", "Feedback" }
                p { class: "card__body",
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
                            strong { "Information" }
                            br {}
                            "Your account settings have been updated."
                        }
                        div { class: "hi-alert hi-alert-warning",
                            strong { "Deprecated" }
                            br {}
                            "This API will be removed in version 2.0."
                        }
                    }
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Inline Status Badges" }
                    div { class: "demo-block__body",
                        div { class: "demo-row",
                            span { class: "badge badge-success", "Online" }
                            span { class: "badge badge-warning", "Pending" }
                            span { class: "badge badge-error", "Offline" }
                            span { class: "badge badge-primary", "Active" }
                            span { class: "badge badge-default", "Inactive" }
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
