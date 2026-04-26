use std::cell::Cell;

use tairitsu_macros::rsx;
use tairitsu_vdom::{MouseEvent, VNode, VText};

thread_local! {
    static CLICK_COUNT: Cell<usize> = const { Cell::new(0) };
}

pub fn render() -> VNode {
    let count = CLICK_COUNT.with(|c| c.get());
    let count_str = format!("clicks: {}", count);
    let count_display = VNode::Text(VText::new(count_str.as_str()));

    rsx! {
        div { id: "page-event-test", class: "hikari-page",
            h2 { class: "page-section__title", "Event Bridge Test" }
            p { class: "page-section__description",
                "Verifies that on_event(\"click\") handlers fire through the WIT bridge. Click the button."
            }

            div { class: "demo-block",
                div { class: "demo-block__body",
                    style: "display: flex; align-items: center; gap: 16px;",
                    button {
                        id: "event-test-btn",
                        class: "hi-button hi-button-primary",
                        onclick: move |_e: MouseEvent| {
                            CLICK_COUNT.with(|c| c.set(c.get() + 1));
                        },
                        "Click Me"
                    }
                    span {
                        id: "event-test-count",
                        style: "font-family: monospace; font-size: 1.2em; font-weight: bold; padding: 4px 12px; background: rgba(22,32,45,0.92); border-radius: 6px;",
                        ..vec![count_display]
                    }
                }
            }

            div { class: "demo-block",
                h3 { class: "demo-block__title", "How It Works" }
                div { class: "demo-block__body",
                    ol {
                        li { "RSX \"onclick:\" compiles to VElement.on_event(\"click\", closure)" }
                        li { "Closure captures thread_local CLICK_COUNT Cell" }
                        li { "On click: Cell increments → next render reads new value" }
                        li { "mount_vnode_to_app() re-renders the full VNode tree" }
                    }
                }
            }

            div { class: "demo-block",
                h3 { class: "demo-block__title", "Status" }
                div { class: "demo-block__body",
                    div {
                        id: "event-test-status",
                        style: "padding: 12px; border-radius: 6px; background: #f0f9ff; border: 1px solid #bae6fd;",
                        "Click the button above. If the counter increments, the event bridge is working."
                    }
                }
            }
        }
    }
}
