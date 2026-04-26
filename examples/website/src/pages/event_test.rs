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
                "Click button → handler increments Cell → modifies button label via WIT."
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
                        style: "font-family: monospace; font-size: 1.1em; font-weight: bold; padding: 6px 14px; background: rgba(22,32,45,0.92); border-radius: 6px;",
                        ..vec![count_display]
                    }
                }
            }

            div { class: "demo-block",
                h3 { class: "demo-block__title", "Result" }
                div { class: "demo-block__body",
                    div {
                        id: "event-test-result",
                        style: "padding: 12px; border-radius: 6px; font-family: monospace; font-size: 0.85em; white-space: pre-wrap; background: #f0f9ff; border: 1px solid #bae6fd;",
                        "Click the button and check:"
                    }
                }
            }
        }
    }
}
