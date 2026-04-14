use crate::components::breadcrumb;
use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

pub fn render() -> VNode {
    rsx! {
        div { id: "page-component-cascader", class: "hikari-page",
            ..vec![breadcrumb(&[("Home", "/"), ("Components", "/components"), ("Layer 2 — Composed", "/components/layer2/cascader"), ("Cascader", "")])]
            section { class: "page-section",
                h2 { class: "page-section__title", "Cascader" }
                p { class: "card__body",
                    "Cascading selection component for hierarchical data like region/city/district pickers."
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Basic Cascader" }
                    div { class: "demo-block__body",
                        div { class: "form-group",
                            label { "Location" }
                            select { option { "Select..." } option { "Asia / China / Beijing" } option { "Europe / Germany / Berlin" } }
                        }
                    }
                }
            }
        }
    }
}
