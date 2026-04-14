use crate::components::breadcrumb;
use tairitsu_macros::rsx;
use tairitsu_vdom::{VElement, VNode, VText};

fn txt(s: &str) -> VNode {
    VNode::Text(VText::new(s))
}
fn el(tag: &str) -> VElement {
    VElement::new(tag)
}

pub fn render() -> VNode {
    let wifi_cb = VNode::Element(el("input").attr("type", "checkbox").attr("checked", "true"));
    let bt_cb = VNode::Element(el("input").attr("type", "checkbox"));
    rsx! {
        div { id: "page-component-switch", class: "hikari-page",
            ..vec![breadcrumb(&[("Home", "/"), ("Components", "/components"), ("Layer 1 — Base", "/components/layer1/switch"), ("Switch", "")])]
            section { class: "page-section",
                h2 { class: "page-section__title", "Switch" }
                p { class: "card__body",
                    "Toggle switch for boolean state control. Supports custom sizes, labels, and disabled state."
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Basic Switches" }
                    div { class: "demo-block__body",
                        div { class: "switch-row",
                            label { class: "switch-label", "Wi-Fi" }
                            label { class: "hi-switch",
                                ..vec![wifi_cb]
                                span { class: "hi-switch-slider" }
                            }
                        }
                        div { class: "switch-row",
                            label { class: "switch-label", "Bluetooth" }
                            label { class: "hi-switch",
                                ..vec![bt_cb]
                                span { class: "hi-switch-slider" }
                            }
                        }
                    }
                }
            }
        }
    }
}
