use crate::components::breadcrumb;
use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

pub fn render() -> VNode {
    rsx! {
        div { id: "page-component-data", class: "hikari-page",
            ..vec![breadcrumb(&[("Home", "/"), ("Components", "/components"), ("Layer 2 — Composed", "/components/layer2/data"), ("Data", "")])]
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
            }
        }
    }
}
