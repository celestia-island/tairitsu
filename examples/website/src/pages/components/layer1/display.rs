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
        div { id: "page-component-display", class: "hikari-page",
            ..vec![breadcrumb(&[("Home", "/"), ("Components", "/components"), ("Layer 1 \u{2014} Base", "/components/layer1/display"), ("Display", "")])]
            section { class: "page-section",
                h2 { class: "page-section__title", "Display" }
                p { class: "page-section__description",
                    "Read-only display components for presenting data: text, numbers, dates, and status indicators."
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Status Display" }
                    div { class: "demo-block__body",
                        div { class: "display-item",
                            span { class: "display-label", "Build Status" }
                            span { class: "display-value hi-status-active", "Active" }
                        }
                        div { class: "display-item",
                            span { class: "display-label", "Deploy Status" }
                            span { class: "display-value hi-status-warning", "Pending" }
                        }
                        div { class: "display-item",
                            span { class: "display-label", "Test Results" }
                            span { class: "display-value hi-status-error", "Failed" }
                        }
                        div { class: "display-item",
                            span { class: "display-label", "Archived" }
                            span { class: "display-value hi-status-inactive", "Inactive" }
                        }
                    }
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Metadata Display" }
                    div { class: "demo-block__body",
                        div { class: "display-item",
                            span { class: "display-label", "Name" }
                            span { class: "display-value", "Tairitsu" }
                        }
                        div { class: "display-item",
                            span { class: "display-label", "Version" }
                            span { class: "display-value", "0.3.0" }
                        }
                        div { class: "display-item",
                            span { class: "display-label", "Runtime" }
                            span { class: "display-value", "wasm32-wasip2" }
                        }
                        div { class: "display-item",
                            span { class: "display-label", "License" }
                            span { class: "display-value", "Apache-2.0" }
                        }
                    }
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Text Styles" }
                    div { class: "demo-block__body",
                        div { class: "text-style-stack",
                            div { class: "text-style-item",
                                div { class: "text-style-label text-style-caption", "Caption" }
                                p { class: "text-style-body text-style-caption", "This is caption text for supplementary information." }
                            }
                            div { class: "text-style-item",
                                div { class: "text-style-label text-style-body-text", "Body" }
                                p { class: "text-style-body text-style-body-text", "This is standard body text for general content." }
                            }
                            div { class: "text-style-item",
                                div { class: "text-style-label text-style-heading", "Heading" }
                                p { class: "text-style-body text-style-heading", "This is a heading style for emphasis." }
                            }
                            div { class: "text-style-item",
                                div { class: "text-style-label text-style-code", "Code" }
                                p { class: "text-style-body text-style-code", "fn render() -> VNode" }
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
                                tr { td { code { "variant" } } td { code { "active | inactive | warning | error" } } td { "Status color variant" } }
                                tr { td { code { "label" } } td { code { "string" } } td { "Display label text" } }
                                tr { td { code { "value" } } td { code { "string | number" } } td { "Display value" } }
                                tr { td { code { "type" } } td { code { "text | code | number" } } td { "Text rendering style" } }
                            }
                        }
                    }
                }
            }
        }
    }
}
