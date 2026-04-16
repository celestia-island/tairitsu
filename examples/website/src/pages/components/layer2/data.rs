use crate::components::breadcrumb;
use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

pub fn render() -> VNode {
    rsx! {
        div { id: "page-component-data", class: "hikari-page",
            ..vec![breadcrumb(&[("Home", "/"), ("Components", "/components"), ("Layer 2 \u{2014} Composed", "/components/layer2/data"), ("Data", "")])]
            section { class: "page-section",
                h2 { class: "page-section__title", "Data" }
                p { class: "card__body",
                    "Data presentation components for displaying structured information with sorting, filtering, and pagination."
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Data Display" }
                    div { class: "demo-block__body",
                        table { class: "hi-table",
                            thead {
                                tr { th { "Name" } th { "Type" } th { "Status" } }
                            }
                            tbody {
                                tr { td { "Tairitsu" } td { "Runtime" } td { span { class: "hi-tag hi-tag-success", "Active" } } }
                                tr { td { "Hikari" } td { "UI Library" } td { span { class: "hi-tag hi-tag-primary", "Stable" } } }
                            }
                        }
                    }
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Statistics Cards" }
                    div { class: "demo-block__body",
                        div { style: "display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:12px;",
                            div { style: "background:rgba(20,110,116,0.1);border:1px solid rgba(20,110,116,0.2);border-radius:8px;padding:16px;",
                                div { style: "font-size:0.8125rem;color:var(--hi-color-text-secondary);", "Total Components" }
                                div { style: "font-size:1.5rem;font-weight:700;color:var(--hi-color-text-primary);margin-top:4px;", "30" }
                                div { style: "font-size:0.75rem;color:var(--hi-color-success);margin-top:4px;", "+5 this week" }
                            }
                            div { style: "background:rgba(255,199,115,0.08);border:1px solid rgba(255,199,115,0.15);border-radius:8px;padding:16px;",
                                div { style: "font-size:0.8125rem;color:var(--hi-color-text-secondary);", "Test Coverage" }
                                div { style: "font-size:1.5rem;font-weight:700;color:var(--hi-color-text-primary);margin-top:4px;", "87%" }
                                div { style: "font-size:0.75rem;color:var(--hi-color-accent);margin-top:4px;", "+3% from last build" }
                            }
                            div { style: "background:rgba(14,184,64,0.08);border:1px solid rgba(14,184,64,0.15);border-radius:8px;padding:16px;",
                                div { style: "font-size:0.8125rem;color:var(--hi-color-text-secondary);", "Build Status" }
                                div { style: "font-size:1.5rem;font-weight:700;color:var(--hi-color-success);margin-top:4px;", "Passing" }
                                div { style: "font-size:0.75rem;color:var(--hi-color-text-disabled);margin-top:4px;", "Last: 2 min ago" }
                            }
                        }
                    }
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Data List" }
                    div { class: "demo-block__body",
                        div { style: "display:flex;flex-direction:column;",
                            div { style: "display:flex;align-items:center;justify-content:space-between;padding:10px 0;border-bottom:1px solid var(--hi-color-border);",
                                div { style: "display:flex;align-items:center;gap:12px;",
                                    div { class: "hi-avatar hi-avatar-sm", style: "background:#e91e63;", "P" }
                                    div {
                                        div { style: "font-size:0.875rem;color:var(--hi-color-text-primary);font-weight:500;", "vdom" }
                                        div { style: "font-size:0.8125rem;color:var(--hi-color-text-disabled);", "Virtual DOM implementation" }
                                    }
                                }
                                span { class: "hi-tag hi-tag-success", "Stable" }
                            }
                            div { style: "display:flex;align-items:center;justify-content:space-between;padding:10px 0;border-bottom:1px solid var(--hi-color-border);",
                                div { style: "display:flex;align-items:center;gap:12px;",
                                    div { class: "hi-avatar hi-avatar-sm", "H" }
                                    div {
                                        div { style: "font-size:0.875rem;color:var(--hi-color-text-primary);font-weight:500;", "hooks" }
                                        div { style: "font-size:0.8125rem;color:var(--hi-color-text-disabled);", "React-like state hooks" }
                                    }
                                }
                                span { class: "hi-tag hi-tag-success", "Stable" }
                            }
                            div { style: "display:flex;align-items:center;justify-content:space-between;padding:10px 0;border-bottom:1px solid var(--hi-color-border);",
                                div { style: "display:flex;align-items:center;gap:12px;",
                                    div { class: "hi-avatar hi-avatar-sm", style: "background:#2196f3;", "M" }
                                    div {
                                        div { style: "font-size:0.875rem;color:var(--hi-color-text-primary);font-weight:500;", "macros" }
                                        div { style: "font-size:0.8125rem;color:var(--hi-color-text-disabled);", "RSX! macro for declarative UI" }
                                    }
                                }
                                span { class: "hi-tag hi-tag-warning", "Beta" }
                            }
                            div { style: "display:flex;align-items:center;justify-content:space-between;padding:10px 0;",
                                div { style: "display:flex;align-items:center;gap:12px;",
                                    div { class: "hi-avatar hi-avatar-sm", style: "background:#4caf50;", "W" }
                                    div {
                                        div { style: "font-size:0.875rem;color:var(--hi-color-text-primary);font-weight:500;", "web" }
                                        div { style: "font-size:0.8125rem;color:var(--hi-color-text-disabled);", "WASI browser bindings" }
                                    }
                                }
                                span { class: "hi-tag hi-tag-primary", "Active" }
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
                                tr { td { code { "data" } } td { code { "T[]" } } td { "Data source array" } }
                                tr { td { code { "columns" } } td { code { "Column[]" } } td { "Column definitions" } }
                                tr { td { code { "sortable" } } td { code { "bool" } } td { "Enable column sorting" } }
                                tr { td { code { "filterable" } } td { code { "bool" } } td { "Enable row filtering" } }
                                tr { td { code { "loading" } } td { code { "bool" } } td { "Show loading state" } }
                            }
                        }
                    }
                }
            }
        }
    }
}
