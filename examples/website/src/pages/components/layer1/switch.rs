use std::cell::{Cell, RefCell};

use crate::app;
use crate::components::breadcrumb;
use tairitsu_macros::rsx;
use tairitsu_vdom::{VElement, VNode};

fn el(tag: &str) -> VElement {
    VElement::new(tag)
}

thread_local! {
    static SWITCH_STATES: RefCell<Vec<Cell<bool>>> = const { RefCell::new(Vec::new()) };
}

fn switch(state_index: usize, _default_checked: bool, extra_class: &str) -> VNode {
    let checked = SWITCH_STATES.with(|s| s.borrow()[state_index].get());
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
    VNode::Element(
        el("label")
            .class(class_str.as_str())
            .on_event("click", move |_event| {
                SWITCH_STATES.with(|states| {
                    if let Some(state) = states.borrow().get(state_index) {
                        state.set(!state.get());
                    }
                });
                app::rerender();
            })
            .children(vec![
                cb,
                VNode::Element(el("span").class("hi-switch-slider")),
            ]),
    )
}

pub fn render() -> VNode {
    SWITCH_STATES.with(|s| {
        let mut states = s.borrow_mut();
        if states.is_empty() {
            for &init in &[true, false, false, true, true, true, true, false] {
                states.push(Cell::new(init));
            }
        }
    });
    rsx! {
        div { id: "page-component-switch", class: "hikari-page",
            ..vec![breadcrumb(&[("Home", "/"), ("Components", "/components"), ("Layer 1 \u{2014} Base", "/components/layer1/switch"), ("Switch", "")])]
            section { class: "page-section",
                h2 { class: "page-section__title", "Switch" }
                p { class: "page-section__description",
                    "Toggle switch for boolean state control. Supports custom sizes, labels, and disabled state."
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Basic Switches" }
                    div { class: "demo-block__body",
                        div { class: "switch-row",
                            label { class: "switch-label", "Wi-Fi" }
                            ..vec![switch(0, true, "")]
                        }
                        div { class: "switch-row",
                            label { class: "switch-label", "Bluetooth" }
                            ..vec![switch(1, false, "")]
                        }
                        div { class: "switch-row",
                            label { class: "switch-label", "Airplane Mode" }
                            ..vec![switch(2, false, "")]
                        }
                    }
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Switch Sizes" }
                    div { class: "demo-block__body",
                        div { class: "switch-row",
                            label { class: "switch-label", "Small" }
                            ..vec![switch(3, true, "hi-switch-sm")]
                        }
                        div { class: "switch-row",
                            label { class: "switch-label", "Default" }
                            ..vec![switch(4, true, "")]
                        }
                        div { class: "switch-row",
                            label { class: "switch-label", "Large" }
                            ..vec![switch(5, true, "hi-switch-lg")]
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
                                ..vec![switch(6, true, "")]
                            }
                            div { class: "switch-desc-row",
                                div { class: "switch-desc-info",
                                    div { class: "switch-desc-title", "Notifications" }
                                    p { class: "switch-desc-subtitle", "Receive push notifications for updates" }
                                }
                                ..vec![switch(7, false, "")]
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
