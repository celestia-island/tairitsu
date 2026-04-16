use crate::components::breadcrumb;
use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

pub fn render() -> VNode {
    rsx! {
        div { id: "page-component-empty", class: "hikari-page",
            ..vec![breadcrumb(&[("Home", "/"), ("Components", "/components"), ("Layer 1 \u{2014} Base", "/components/layer1/empty"), ("Empty", "")])]
            section { class: "page-section",
                h2 { class: "page-section__title", "Empty" }
                p { class: "card__body",
                    "Placeholder state for empty data lists with customisable description and action button."
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Basic Empty State" }
                    div { class: "demo-block__body",
                        div { class: "hi-empty",
                            div { class: "hi-empty-icon", "\u{1F4E6}" }
                            p { class: "hi-empty-description", "No data available" }
                            a { href: "#", class: "hi-button hi-button-primary", "Create Now" }
                        }
                    }
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Search Not Found" }
                    div { class: "demo-block__body",
                        div { class: "hi-empty",
                            div { class: "hi-empty-icon", "\u{1F50E}" }
                            p { class: "hi-empty-description", "No results found for your search query." }
                            a { href: "#", class: "hi-button hi-button-secondary", "Clear Filters" }
                        }
                    }
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Empty Table" }
                    div { class: "demo-block__body",
                        table { class: "hi-table",
                            thead {
                                tr { th { "Name" } th { "Status" } th { "Actions" } }
                            }
                            tbody {
                                tr {
                                    td { colspan: "3",
                                        div { class: "hi-empty",
                                            div { class: "hi-empty-icon", style: "font-size:2rem;", "\u{1F4C1}" }
                                            p { class: "hi-empty-description", style: "margin-bottom:0.5rem;", "No files uploaded yet" }
                                            a { href: "#", class: "hi-button hi-button-primary", style: "padding:4px 12px;font-size:0.75rem;", "Upload" }
                                        }
                                    }
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
                                tr { td { code { "image" } } td { code { "string" } } td { "Empty state image URL" } }
                                tr { td { code { "description" } } td { code { "string" } } td { "Description text" } }
                                tr { td { code { "action" } } td { code { "VNode" } } td { "Action button element" } }
                            }
                        }
                    }
                }
            }
        }
    }
}
