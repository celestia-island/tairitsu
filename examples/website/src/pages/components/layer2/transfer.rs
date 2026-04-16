use crate::components::breadcrumb;
use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

pub fn render() -> VNode {
    rsx! {
        div { id: "page-component-transfer", class: "hikari-page",
            ..vec![breadcrumb(&[("Home", "/"), ("Components", "/components"), ("Layer 2 \u{2014} Composed", "/components/layer2/transfer"), ("Transfer", "")])]
            section { class: "page-section",
                h2 { class: "page-section__title", "Transfer" }
                p { class: "card__body",
                    "Dual-list transfer component for moving items between source and target lists with search and sorting."
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Basic Transfer" }
                    div { class: "demo-block__body",
                        div { style: "display:flex;gap:16px;align-items:flex-start;flex-wrap:wrap;",
                            div { class: "transfer-panel",
                                h4 { "Available" }
                                ul {
                                    li { "Option A" }
                                    li { "Option B" }
                                    li { "Option C" }
                                }
                            }
                            div { class: "transfer-controls",
                                button { class: "hi-button", style: "padding:6px 12px;font-size:0.875rem;", "\u{2192}" }
                                button { class: "hi-button", style: "padding:6px 12px;font-size:0.875rem;", "\u{2190}" }
                            }
                            div { class: "transfer-panel",
                                h4 { "Selected" }
                                ul { li { "Option D" } }
                            }
                        }
                    }
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Transfer with Selection" }
                    div { class: "demo-block__body",
                        div { style: "display:flex;gap:16px;align-items:flex-start;flex-wrap:wrap;",
                            div { class: "transfer-panel",
                                h4 { "Framework Features (4)" }
                                ul {
                                    li { class: "selected", "\u{2713} Virtual DOM" }
                                    li { class: "selected", "\u{2713} Reactive Hooks" }
                                    li { "RSX Macros" }
                                    li { "WASI Runtime" }
                                }
                            }
                            div { class: "transfer-controls",
                                button { class: "hi-button hi-button-primary", style: "padding:6px 12px;font-size:0.875rem;", "\u{2192}" }
                                button { class: "hi-button hi-button-secondary", style: "padding:6px 12px;font-size:0.875rem;", "\u{2190}" }
                            }
                            div { class: "transfer-panel",
                                h4 { "Enabled (2)" }
                                ul {
                                    li { "\u{2713} Virtual DOM" }
                                    li { "\u{2713} Reactive Hooks" }
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
                                tr { td { code { "dataSource" } } td { code { "TransferItem[]" } } td { "Transfer data items" } }
                                tr { td { code { "targetKeys" } } td { code { "string[]" } } td { "Keys of selected items" } }
                                tr { td { code { "searchable" } } td { code { "bool" } } td { "Enable search in panels" } }
                                tr { td { code { "showSelectAll" } } td { code { "bool" } } td { "Show select all checkbox" } }
                                tr { td { code { "onChange" } } td { code { "(targetKeys) => void" } } td { "Selection change callback" } }
                            }
                        }
                    }
                }
            }
        }
    }
}
