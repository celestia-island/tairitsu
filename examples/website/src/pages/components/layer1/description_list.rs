use crate::components::breadcrumb;
use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

pub fn render() -> VNode {
    rsx! {
        div { id: "page-component-description-list", class: "hikari-page",
            ..vec![breadcrumb(&[("Home", "/"), ("Components", "/components"), ("Layer 1 \u{2014} Base", "/components/layer1/description-list"), ("Description List", "")])]
            section { class: "page-section",
                h2 { class: "page-section__title", "Description List" }
                p { class: "page-section__description",
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
                            dt { class: "hi-dl-term", "License" }
                            dd { class: "hi-dl-detail", "Apache-2.0" }
                        }
                    }
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Project Metadata" }
                    div { class: "demo-block__body",
                        dl { class: "hi-description-list",
                            dt { class: "hi-dl-term", "Package" }
                            dd { class: "hi-dl-detail", "tairitsu-website" }
                            dt { class: "hi-dl-term", "Edition" }
                            dd { class: "hi-dl-detail", "2024" }
                            dt { class: "hi-dl-term", "Build Target" }
                            dd { class: "hi-dl-detail", "wasm32-wasip2 (component)" }
                            dt { class: "hi-dl-term", "CSS Framework" }
                            dd { class: "hi-dl-detail", "Hikari Design System" }
                            dt { class: "hi-dl-term", "Optimization" }
                            dd { class: "hi-dl-detail", "Enabled" }
                            dt { class: "hi-dl-term", "Sourcemap" }
                            dd { class: "hi-dl-detail", "Enabled" }
                        }
                    }
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "With Tags and Status" }
                    div { class: "demo-block__body",
                        dl { class: "hi-description-list",
                            dt { class: "hi-dl-term", "Status" }
                            dd { class: "hi-dl-detail", span { class: "hi-badge hi-badge-success", "Active" } }
                            dt { class: "hi-dl-term", "Priority" }
                            dd { class: "hi-dl-detail", span { class: "hi-badge hi-badge-warning", "Medium" } }
                            dt { class: "hi-dl-term", "Assignee" }
                            dd { class: "hi-dl-detail", "Tairitsu Team" }
                            dt { class: "hi-dl-term", "Labels" }
                            dd { class: "hi-dl-detail",
                                div { style: "display:flex;gap:4px;flex-wrap:wrap;",
                                    span { class: "hi-tag hi-tag-primary", "framework" }
                                    span { class: "hi-tag hi-tag-default", "wasm" }
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
                                tr { td { code { "term" } } td { code { "string" } } td { "Key/label text" } }
                                tr { td { code { "detail" } } td { code { "string | VNode" } } td { "Value content" } }
                                tr { td { code { "layout" } } td { code { "horizontal | vertical" } } td { "Term-detail alignment" } }
                                tr { td { code { "colon" } } td { code { "bool" } } td { "Show colon after term" } }
                            }
                        }
                    }
                }
            }
        }
    }
}
