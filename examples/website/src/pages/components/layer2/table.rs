use crate::components::breadcrumb;
use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

pub fn render() -> VNode {
    rsx! {
        div { id: "page-component-table", class: "hikari-page",
            ..vec![breadcrumb(&[("Home", "/"), ("Components", "/components"), ("Layer 2 — Composed", "/components/layer2/table"), ("Table", "")])]
            section { class: "page-section",
                h2 { class: "page-section__title", "Table" }
                p { class: "card__body",
                    "Full-featured data table with sorting, selection, fixed columns, row expansion, and virtual scrolling."
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Basic Table" }
                    div { class: "demo-block__body",
                        table { class: "hi-table hi-table-bordered",
                            thead {
                                tr { th { "ID" } th { "Name" } th { "Category" } th { "Actions" } }
                            }
                            tbody {
                                tr { td { "1" } td { "Button" } td { "Base" } td { a { href: "#", class: "hi-button-link", "View" } } }
                                tr { td { "2" } td { "Form" } td { "Base" } td { a { href: "#", class: "hi-button-link", "View" } } }
                                tr { td { "3" } td { "Table" } td { "Composed" } td { a { href: "#", class: "hi-button-link", "View" } } }
                            }
                        }
                    }
                }
            }
        }
    }
}
