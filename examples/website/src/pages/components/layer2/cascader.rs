use crate::components::breadcrumb;
use tairitsu_macros::rsx;
use tairitsu_vdom::{VElement, VNode, VText};

fn el(tag: &str) -> VElement {
    VElement::new(tag)
}
fn txt(s: &str) -> VNode {
    VNode::Text(VText::new(s))
}

pub fn render() -> VNode {
    rsx! {
        div { id: "page-component-cascader", class: "hikari-page",
            ..vec![breadcrumb(&[("Home", "/"), ("Components", "/components"), ("Layer 2 \u{2014} Composed", "/components/layer2/cascader"), ("Cascader", "")])]
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
                            div { class: "form-input-wrapper",
                                ..vec![VNode::Element(el("select").children(vec![
                                    VNode::Element(el("option").child(txt("Select location..."))),
                                    VNode::Element(el("option").child(txt("Asia / China / Beijing"))),
                                    VNode::Element(el("option").child(txt("Asia / China / Shanghai"))),
                                    VNode::Element(el("option").child(txt("Asia / Japan / Tokyo"))),
                                    VNode::Element(el("option").child(txt("Europe / Germany / Berlin"))),
                                    VNode::Element(el("option").child(txt("Europe / France / Paris"))),
                                    VNode::Element(el("option").child(txt("North America / USA / San Francisco"))),
                                ]))]
                            }
                        }
                    }
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Cascader Steps Display" }
                    div { class: "demo-block__body",
                        div { style: "display:flex;flex-direction:column;gap:16px;",
                            div { style: "display:flex;align-items:center;gap:8px;",
                                div { style: "display:flex;gap:4px;",
                                    span { class: "hi-tag hi-tag-primary", "Asia" }
                                    span { style: "color:var(--hi-color-text-disabled);", ">" }
                                    span { class: "hi-tag hi-tag-primary", "China" }
                                    span { style: "color:var(--hi-color-text-disabled);", ">" }
                                    span { class: "hi-tag hi-tag-primary", "Beijing" }
                                }
                            }
                            div { style: "display:flex;align-items:center;gap:8px;",
                                div { style: "display:flex;gap:4px;",
                                    span { class: "hi-tag hi-tag-primary", "Technology" }
                                    span { style: "color:var(--hi-color-text-disabled);", ">" }
                                    span { class: "hi-tag hi-tag-primary", "Programming" }
                                    span { style: "color:var(--hi-color-text-disabled);", ">" }
                                    span { class: "hi-tag hi-tag-warning", "Select..." }
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
                                tr { td { code { "options" } } td { code { "CascaderOption[]" } } td { "Hierarchical options data" } }
                                tr { td { code { "value" } } td { code { "string[]" } } td { "Selected value path" } }
                                tr { td { code { "multiple" } } td { code { "bool" } } td { "Allow multiple selections" } }
                                tr { td { code { "searchable" } } td { code { "bool" } } td { "Enable search filtering" } }
                                tr { td { code { "onChange" } } td { code { "(value: string[]) => void" } } td { "Selection change callback" } }
                                tr { td { code { "loadData" } } td { code { "(node) => Promise" } } td { "Async child loading" } }
                            }
                        }
                    }
                }
            }
        }
    }
}
