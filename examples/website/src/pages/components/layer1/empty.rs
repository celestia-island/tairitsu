use crate::components::{breadcrumb, svg_icon};
use hikari_icons::MdiIcon;
use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

pub fn render() -> VNode {
    rsx! {
        div { id: "page-component-empty", class: "hikari-page",
            ..vec![breadcrumb(&[("Home", "/"), ("Components", "/components"), ("Layer 1 \u{2014} Base", "/components/layer1/empty"), ("Empty", "")])]
            section { class: "page-section",
                h2 { class: "page-section__title", "Empty" }
                p { class: "page-section__description",
                    "Placeholder state for empty data lists with customisable description and action button."
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Basic Empty State" }
                    div { class: "demo-block__body",
                        div { class: "hi-empty",
                            div { class: "hi-empty-icon", ..vec![svg_icon(MdiIcon::Package, 48, "")] }
                            p { class: "hi-empty-description", "No data available" }
                            a { href: "#", class: "hi-button hi-button-primary", "Create Now" }
                        }
                    }
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Search Not Found" }
                    div { class: "demo-block__body",
                        div { class: "hi-empty",
                            div { class: "hi-empty-icon", ..vec![svg_icon(MdiIcon::MagnifyPlus, 48, "")] }
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
                                            div { class: "hi-empty-icon", ..vec![svg_icon(MdiIcon::Package, 32, "")] }
                                            p { class: "hi-empty-description", style: "margin-bottom:0.5rem;", "No files uploaded yet" }
                                            a { href: "#", class: "hi-button hi-button-primary hi-button-sm", "Upload" }
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
