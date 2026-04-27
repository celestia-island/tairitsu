use crate::components::breadcrumb;
use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

pub fn render() -> VNode {
    rsx! {
        div { id: "page-component-tag", class: "hikari-page",
            ..vec![breadcrumb(&[("Home", "/"), ("Components", "/components"), ("Layer 1 \u{2014} Base", "/components/layer1/tag"), ("Tag", "")])]
            section { class: "page-section",
                h2 { class: "page-section__title", "Tag" }
                p { class: "card__body",
                    "Labeling and categorisation component with color variants, closable option, and icon support."
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Tag Variants" }
                    div { class: "demo-block__body",
                        div { class: "demo-row",
                            span { class: "hi-tag hi-tag-primary", "Primary" }
                            span { class: "hi-tag hi-tag-success", "Success" }
                            span { class: "hi-tag hi-tag-warning", "Warning" }
                            span { class: "hi-tag hi-tag-error", "Error" }
                            span { class: "hi-tag hi-tag-default", "Default" }
                        }
                    }
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Tag Usage Examples" }
                    div { class: "demo-block__body",
                        div { style: "margin-bottom:8px;",
                            span { style: "font-size:0.8125rem;color:var(--hi-color-text-secondary);", "Status: " }
                            span { class: "hi-tag hi-tag-success", "Published" }
                        }
                        div { style: "margin-bottom:8px;",
                            span { style: "font-size:0.8125rem;color:var(--hi-color-text-secondary);", "Priority: " }
                            span { class: "hi-tag hi-tag-error", "Urgent" }
                        }
                        div { style: "margin-bottom:8px;",
                            span { style: "font-size:0.8125rem;color:var(--hi-color-text-secondary);", "Category: " }
                            span { class: "hi-tag hi-tag-primary", "Framework" }
                            span { class: "hi-tag hi-tag-default", "v0.3.0" }
                        }
                        div { style: "margin-bottom:8px;",
                            span { style: "font-size:0.8125rem;color:var(--hi-color-text-secondary);", "Labels: " }
                            span { class: "hi-tag hi-tag-primary", "wasm" }
                            span { class: "hi-tag hi-tag-warning", "experimental" }
                            span { class: "hi-tag hi-tag-default", "rust" }
                            span { class: "hi-tag hi-tag-default", "component-model" }
                        }
                    }
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Tag in a List" }
                    div { class: "demo-block__body",
                        div { style: "display:flex;flex-direction:column;gap:8px;",
                            div { style: "display:flex;align-items:center;justify-content:space-between;padding:8px 0;border-bottom:1px solid var(--hi-color-border);",
                                div { style: "font-size:0.875rem;color:var(--hi-color-text-primary);", "Component Library" }
                                span { class: "hi-tag hi-tag-success", "Stable" }
                            }
                            div { style: "display:flex;align-items:center;justify-content:space-between;padding:8px 0;border-bottom:1px solid var(--hi-color-border);",
                                div { style: "font-size:0.875rem;color:var(--hi-color-text-primary);", "Runtime Engine" }
                                span { class: "hi-tag hi-tag-warning", "Beta" }
                            }
                            div { style: "display:flex;align-items:center;justify-content:space-between;padding:8px 0;",
                                div { style: "font-size:0.875rem;color:var(--hi-color-text-primary);", "Visual Editor" }
                                span { class: "hi-tag hi-tag-primary", "Planned" }
                            }
                        }
                    }
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Closable Tag" }
                    div { class: "demo-block__body",
                        div { class: "demo-row",
                            span { class: "hi-tag hi-tag-closable hi-tag-primary",
                                "Closable"
                                svg { class: "hi-tag-close-btn", xmlns: "http://www.w3.org/2000/svg", viewBox: "0 0 24 24", width: "14", height: "14", fill: "none", stroke: "currentColor", "stroke-width": "2",
                                    path { d: "M18 6L6 18M6 6l12 12" }
                                }
                            }
                            span { class: "hi-tag hi-tag-closable hi-tag-success",
                                "Published"
                                svg { class: "hi-tag-close-btn", xmlns: "http://www.w3.org/2000/svg", viewBox: "0 0 24 24", width: "14", height: "14", fill: "none", stroke: "currentColor", "stroke-width": "2",
                                    path { d: "M18 6L6 18M6 6l12 12" }
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
                                tr { td { code { "variant" } } td { code { "primary | success | warning | error | default" } } td { "Tag color" } }
                                tr { td { code { "closable" } } td { code { "bool" } } td { "Show close button" } }
                                tr { td { code { "icon" } } td { code { "VNode" } } td { "Leading icon" } }
                                tr { td { code { "onClose" } } td { code { "() => void" } } td { "Close callback" } }
                            }
                        }
                    }
                }
            }
        }
    }
}
