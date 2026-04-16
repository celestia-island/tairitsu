use crate::components::breadcrumb;
use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

pub fn render() -> VNode {
    rsx! {
        div { id: "page-component-display", class: "hikari-page",
            ..vec![breadcrumb(&[("Home", "/"), ("Components", "/components"), ("Layer 1 \u{2014} Base", "/components/layer1/display"), ("Display", "")])]
            section { class: "page-section",
                h2 { class: "page-section__title", "Display" }
                p { class: "card__body",
                    "Read-only display components for presenting data: text, numbers, dates, and status indicators."
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Status Display" }
                    div { class: "demo-block__body",
                        div { class: "display-item",
                            span { class: "display-label", "Build Status" }
                            span { class: "display-value hi-status-active", "\u{25CF} Active" }
                        }
                        div { class: "display-item",
                            span { class: "display-label", "Deploy Status" }
                            span { class: "display-value hi-status-warning", "\u{25CF} Pending" }
                        }
                        div { class: "display-item",
                            span { class: "display-label", "Test Results" }
                            span { class: "display-value hi-status-error", "\u{25CF} Failed" }
                        }
                        div { class: "display-item",
                            span { class: "display-label", "Archived" }
                            span { class: "display-value hi-status-inactive", "\u{25CF} Inactive" }
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
                        div { style: "display:flex;flex-direction:column;gap:12px;",
                            div {
                                div { style: "font-size:0.75rem;color:var(--hi-color-text-disabled);margin-bottom:2px;", "Caption" }
                                p { style: "font-size:0.75rem;color:var(--hi-color-text-disabled);margin:0;", "This is caption text for supplementary information." }
                            }
                            div {
                                div { style: "font-size:0.875rem;color:var(--hi-color-text-secondary);margin-bottom:2px;", "Body" }
                                p { style: "font-size:0.875rem;color:var(--hi-color-text-secondary);margin:0;", "This is standard body text for general content." }
                            }
                            div {
                                div { style: "font-size:1rem;font-weight:600;color:var(--hi-color-text-primary);margin-bottom:2px;", "Heading" }
                                p { style: "font-size:1rem;font-weight:600;color:var(--hi-color-text-primary);margin:0;", "This is a heading style for emphasis." }
                            }
                            div {
                                div { style: "font-family:var(--ts-font-mono);font-size:0.875rem;color:var(--hi-color-secondary);margin-bottom:2px;", "Code" }
                                p { style: "font-family:var(--ts-font-mono);font-size:0.875rem;color:var(--hi-color-secondary);margin:0;", "fn render() -> VNode" }
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
