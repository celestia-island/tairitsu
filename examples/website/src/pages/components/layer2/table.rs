use crate::components::breadcrumb;
use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

pub fn render() -> VNode {
    rsx! {
        div { id: "page-component-table", class: "hikari-page",
            ..vec![breadcrumb(&[("Home", "/"), ("Components", "/components"), ("Layer 2 \u{2014} Composed", "/components/layer2/table"), ("Table", "")])]
            section { class: "page-section",
                h2 { class: "page-section__title", "Table" }
                p { class: "card__body",
                    "Full-featured data table with sorting, selection, fixed columns, row expansion, and virtual scrolling."
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Basic Table" }
                    div { class: "demo-block__body",
                        table { class: "hi-table",
                            thead {
                                tr { th { "ID" } th { "Name" } th { "Category" } th { "Status" } }
                            }
                            tbody {
                                tr { td { "1" } td { "Button" } td { "Base" } td { span { class: "hi-badge hi-badge-success", "Stable" } } }
                                tr { td { "2" } td { "Form" } td { "Base" } td { span { class: "hi-badge hi-badge-success", "Stable" } } }
                                tr { td { "3" } td { "Table" } td { "Composed" } td { span { class: "hi-badge hi-badge-success", "Stable" } } }
                                tr { td { "4" } td { "Editor" } td { "Complex" } td { span { class: "hi-badge hi-badge-warning", "Beta" } } }
                            }
                        }
                    }
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Bordered Table" }
                    div { class: "demo-block__body",
                        table { class: "hi-table hi-table-bordered",
                            thead {
                                tr { th { "Package" } th { "Version" } th { "Description" } }
                            }
                            tbody {
                                tr { td { "tairitsu-vdom" } td { "0.3.0" } td { "Virtual DOM implementation" } }
                                tr { td { "tairitsu-hooks" } td { "0.3.0" } td { "Reactive state hooks" } }
                                tr { td { "tairitsu-macros" } td { "0.3.0" } td { "RSX! declarative UI macro" } }
                            }
                        }
                    }
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Striped & Hoverable Table" }
                    div { class: "demo-block__body",
                        table { class: "hi-table hi-table-striped hi-table-hover",
                            thead {
                                tr { th { "Component" } th { "Layer" } th { "Actions" } }
                            }
                            tbody {
                                tr { td { "Button" } td { "Layer 1" } td { div { class: "table-actions", a { href: "#", class: "hi-button-link", "View" } a { href: "#", class: "hi-button-link", "Edit" } } } }
                                tr { td { "Navigation" } td { "Layer 2" } td { div { class: "table-actions", a { href: "#", class: "hi-button-link", "View" } a { href: "#", class: "hi-button-link", "Edit" } } } }
                                tr { td { "Media" } td { "Layer 3" } td { div { class: "table-actions", a { href: "#", class: "hi-button-link", "View" } a { href: "#", class: "hi-button-link", "Edit" } } } }
                                tr { td { "Timeline" } td { "Layer 2" } td { div { class: "table-actions", a { href: "#", class: "hi-button-link", "View" } a { href: "#", class: "hi-button-link", "Edit" } } } }
                                tr { td { "Editor" } td { "Layer 3" } td { div { class: "table-actions", a { href: "#", class: "hi-button-link", "View" } a { href: "#", class: "hi-button-link", "Edit" } } } }
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
                                tr { td { code { "data" } } td { code { "T[]" } } td { "Table data source" } }
                                tr { td { code { "columns" } } td { code { "Column[]" } } td { "Column definitions" } }
                                tr { td { code { "bordered" } } td { code { "bool" } } td { "Show cell borders" } }
                                tr { td { code { "striped" } } td { code { "bool" } } td { "Alternate row background" } }
                                tr { td { code { "hoverable" } } td { code { "bool" } } td { "Row hover highlight" } }
                                tr { td { code { "selectable" } } td { code { "bool" } } td { "Enable row selection" } }
                                tr { td { code { "loading" } } td { code { "bool" } } td { "Show loading skeleton" } }
                            }
                        }
                    }
                }
            }
        }
    }
}
