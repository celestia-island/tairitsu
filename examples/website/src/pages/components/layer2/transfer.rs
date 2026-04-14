use crate::components::breadcrumb;
use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

pub fn render() -> VNode {
    rsx! {
        div { id: "page-component-transfer", class: "hikari-page",
            ..vec![breadcrumb(&[("Home", "/"), ("Components", "/components"), ("Layer 2 — Composed", "/components/layer2/transfer"), ("Transfer", "")])]
            section { class: "page-section",
                h2 { class: "page-section__title", "Transfer" }
                p { class: "card__body",
                    "Dual-list transfer component for moving items between source and target lists with search and sorting."
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Basic Transfer" }
                    div { class: "demo-block__body",
                        div { style: "display:flex;gap:16px;align-items:flex-start;",
                            div { class: "transfer-panel",
                                h4 { "Available" }
                                ul { li { "Option A" } li { "Option B" } li { "Option C" } }
                            }
                            div { class: "transfer-controls",
                                button { class: "hi-button", "\u{2192}" }
                                br {}
                                button { class: "hi-button", "\u{2190}" }
                            }
                            div { class: "transfer-panel",
                                h4 { "Selected" }
                                ul { li { "Option D" } }
                            }
                        }
                    }
                }
            }
        }
    }
}
