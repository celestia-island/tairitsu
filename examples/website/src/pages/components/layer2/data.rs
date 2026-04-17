use crate::components::breadcrumb;
use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

pub fn render() -> VNode {
    rsx! {
        div { id: "page-component-data", class: "hikari-page",
            ..vec![breadcrumb(&[("Home", "/"), ("Components", "/components"), ("Layer 2 \u{2014} Composed", "/components/layer2/data"), ("Data", "")])]
            section { class: "page-section",
                h2 { class: "page-section__title", "Data" }
                p { class: "page-section__description",
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
                        div { class: "data-stats-grid",
                            div { class: "data-stat-card data-stat-card--primary",
                                div { class: "data-stat-label", "Total Components" }
                                div { class: "data-stat-value", "30" }
                                div { class: "data-stat-trend data-stat-trend--up", "+5 this week" }
                            }
                            div { class: "data-stat-card data-stat-card--warning",
                                div { class: "data-stat-label", "Test Coverage" }
                                div { class: "data-stat-value", "87%" }
                                div { class: "data-stat-trend data-stat-trend--up", "+3% from last build" }
                            }
                            div { class: "data-stat-card data-stat-card--success",
                                div { class: "data-stat-label", "Build Status" }
                                div { class: "data-stat-value", "Passing" }
                                div { class: "data-stat-trend data-stat-trend--neutral", "Last: 2 min ago" }
                            }
                        }
                    }
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Data List" }
                    div { class: "demo-block__body",
                        div { class: "data-list",
                            div { class: "data-list-item",
                                div { class: "data-list-item-avatar",
                                    div { class: "hi-avatar hi-avatar-sm", style: "background:#e91e63;", "P" }
                                }
                                div { class: "data-list-item-content",
                                    div { class: "data-list-item-name", "vdom" }
                                    div { class: "data-list-item-desc", "Virtual DOM implementation" }
                                }
                                span { class: "hi-tag hi-tag-success", "Stable" }
                            }
                            div { class: "data-list-item",
                                div { class: "data-list-item-avatar",
                                    div { class: "hi-avatar hi-avatar-sm", "H" }
                                }
                                div { class: "data-list-item-content",
                                    div { class: "data-list-item-name", "hooks" }
                                    div { class: "data-list-item-desc", "React-like state hooks" }
                                }
                                span { class: "hi-tag hi-tag-success", "Stable" }
                            }
                            div { class: "data-list-item",
                                div { class: "data-list-item-avatar",
                                    div { class: "hi-avatar hi-avatar-sm", style: "background:#2196f3;", "M" }
                                }
                                div { class: "data-list-item-content",
                                    div { class: "data-list-item-name", "macros" }
                                    div { class: "data-list-item-desc", "RSX! macro for declarative UI" }
                                }
                                span { class: "hi-tag hi-tag-warning", "Beta" }
                            }
                            div { class: "data-list-item data-list-item--last",
                                div { class: "data-list-item-avatar",
                                    div { class: "hi-avatar hi-avatar-sm", style: "background:#4caf50;", "W" }
                                }
                                div { class: "data-list-item-content",
                                    div { class: "data-list-item-name", "web" }
                                    div { class: "data-list-item-desc", "WASI browser bindings" }
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
