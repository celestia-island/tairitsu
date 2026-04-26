use crate::components::breadcrumb;
use std::cell::Cell;
use tairitsu_macros::rsx;
use tairitsu_vdom::{VElement, VNode};

thread_local! {
    static WIFI_STATE: Cell<bool> = const { Cell::new(true) };
}

fn el(tag: &str) -> VElement {
    VElement::new(tag)
}
fn switch(checked: bool, extra_class: &str) -> VNode {
    let cb = VNode::Element(if checked {
        el("input").attr("type", "checkbox").attr("checked", "true")
    } else {
        el("input").attr("type", "checkbox")
    });
    let class_str = if extra_class.is_empty() {
        "hi-switch".to_string()
    } else {
        format!("hi-switch {}", extra_class)
    };
    VNode::Element(el("label").class(class_str.as_str()).children(vec![
        cb,
        VNode::Element(el("span").class("hi-switch-slider")),
    ]))
}
fn interactive_switch(checked: bool, extra_class: &str) -> VNode {
    let cb = VNode::Element(if checked {
        el("input").attr("type", "checkbox").attr("checked", "true")
    } else {
        el("input").attr("type", "checkbox")
    });
    let class_str = if extra_class.is_empty() {
        "hi-switch".to_string()
    } else {
        format!("hi-switch {}", extra_class)
    };
    VNode::Element(el("label").class(class_str.as_str()).on_event("click", move |_e: Box<dyn tairitsu_vdom::EventData>| {
        WIFI_STATE.with(|c| c.set(!c.get()));
        tairitsu_vdom::rerender();
    }).children(vec![
        cb,
        VNode::Element(el("span").class("hi-switch-slider")),
    ]))
}

pub fn render() -> VNode {
    let wifi_checked = WIFI_STATE.with(|c| c.get());
    rsx! {
        div { id: "page-component-switch", class: "hikari-page",
            ..vec![breadcrumb(&[("Home", "/"), ("Components", "/"), ("Layer 1 \u{2014} Base", "/components/layer1/switch"), ("Switch", "")])]
            section { class: "page-section",
                h2 { class: "page-section__title", "Switch" }
                p { class: "page-section__description",
                    "Toggle switch for boolean state control. Supports custom sizes, labels, and disabled state."
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Basic Switches" }
                    div { class: "demo-block__body",
                        div { class: "switch-row",
                            label { class: "switch-label", id: "switch-wifi", "Wi-Fi" }
                            ..vec![interactive_switch(wifi_checked, "")]
                        }
                        div { class: "switch-row",
                            label { class: "switch-label", "Bluetooth" }
                            ..vec![switch(false, "")]
                        }
                        div { class: "switch-row",
                            label { class: "switch-label", "Airplane Mode" }
                            ..vec![switch(false, "")]
                        }
                    }
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Switch Sizes" }
                    div { class: "demo-block__body",
                        div { class: "switch-row",
                            label { class: "switch-label", "Small" }
                            ..vec![switch(true, "hi-switch-sm")]
                        }
                        div { class: "switch-row",
                            label { class: "switch-label", "Default" }
                            ..vec![switch(true, "")]
                        }
                        div { class: "switch-row",
                            label { class: "switch-label", "Large" }
                            ..vec![switch(true, "hi-switch-lg")]
                        }
                    }
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Switch with Description" }
                    div { class: "demo-block__body",
                        div { class: "switch-desc-list",
                            div { class: "switch-desc-row",
                                div { class: "switch-desc-info",
                                    div { class: "switch-desc-title", "Auto-save" }
                                    p { class: "switch-desc-subtitle", "Automatically save changes every 30 seconds" }
                                }
                                ..vec![switch(true, "")]
                            }
                            div { class: "switch-desc-row",
                                div { class: "switch-desc-info",
                                    div { class: "switch-desc-title", "Notifications" }
                                    p { class: "switch-desc-subtitle", "Receive push notifications for updates" }
                                }
                                ..vec![switch(false, "")]
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
                                tr { td { code { "checked" } } td { code { "bool" } } td { "Controlled checked state" } }
                                tr { td { code { "disabled" } } td { code { "bool" } } td { "Disable the switch" } }
                                tr { td { code { "size" } } td { code { "small | default | large" } } td { "Switch size" } }
                                tr { td { code { "onChange" } } td { code { "(checked: bool) => void" } } td { "Change callback" } }
                            }
                        }
                    }
                }
            }
        }
    }
}
