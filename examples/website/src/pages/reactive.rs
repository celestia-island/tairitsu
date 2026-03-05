use tairitsu_hooks::{use_effect, use_signal, use_state};
use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

pub fn reactive() -> VNode {
    let count = use_signal(0);
    let text = use_state(String::new);

    rsx! {
        div {
            class: "page reactive-demo",
            h1 {
                "Reactive System Demo"
            }

            section {
                class: "demo-section",
                h2 {
                    "1. use_signal - Reactive Signals"
                }
                p {
                    "Signals provide fine-grained reactivity"
                }
                div {
                    class: "demo-box",
                    p {
                        format!("Signal Count: {}", count.get())
                    }
                    button {
                        onclick: move |_| count.set(count.get() + 1),
                        "+1"
                    }
                    button {
                        onclick: move |_| count.set(count.get() - 1),
                        "-1"
                    }
                    button {
                        onclick: move |_| count.set(0),
                        "Reset"
                    }
                }
                pre {
                    code {
                        r#"
let count = use_signal(0);

// Reading
let value = count.get();

// Writing
count.set(42);
count.update(|v| v + 1);
"#
                    }
                }
            }

            section {
                class: "demo-section",
                h2 {
                    "2. use_state - Local State"
                }
                p {
                    "State management with immutable updates"
                }
                div {
                    class: "demo-box",
                    input {
                        type: "text",
                        placeholder: "Type something...",
                        value: text.get(),
                        oninput: move |e| {
                            if let Some(input) = e.target().and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok()) {
                                text.set(input.value());
                            }
                        }
                    }
                    p {
                        format!("You typed: {}", text.get())
                    }
                }
                pre {
                    code {
                        r#"
let text = use_state(String::new);

// Reading
let value = text.get();

// Writing
text.set("new value".to_string());
"#
                    }
                }
            }

            section {
                class: "demo-section",
                h2 {
                    "3. use_effect - Side Effects"
                }
                p {
                    "Run side effects when dependencies change"
                }
                div {
                    class: "demo-box",
                    p {
                        "Check the console for effect logs"
                    }
                    button {
                        onclick: move |_| count.set(count.get() + 1),
                        format!("Count: {} - Click to trigger effect", count.get())
                    }
                }
                pre {
                    code {
                        r#"
let count = use_signal(0);

use_effect(move || {
    // This runs whenever count changes
    web_sys::console::log_1(&format!("Count changed to: {}", count.get()).into());
});
"#
                    }
                }
            }

            section {
                class: "demo-section",
                h2 {
                    "4. Performance: Batch Updates"
                }
                p {
                    "Multiple updates are batched automatically"
                }
                div {
                    class: "demo-box",
                    p {
                        "Updates happen efficiently without unnecessary re-renders"
                    }
                }
                pre {
                    code {
                        r#"
use tairitsu_vdom::batch;

batch(|| {
    // All these updates happen in one batch
    signal1.set(1);
    signal2.set(2);
    signal3.set(3);
});
"#
                    }
                }
            }

            section {
                class: "demo-section",
                h2 {
                    "5. Integration with Builders"
                }
                p {
                    "Reactive values can be used with StyleBuilder and ClassesBuilder"
                }
                div {
                    class: "demo-box",
                    div {
                        style: format!(
                            "background: hsl({}, 70%, 60%); padding: 20px; border-radius: 8px; color: white;",
                            (count.get() * 30) % 360
                        ),
                        "Color changes with count"
                    }
                }
                pre {
                    code {
                        r#"
// Reactive styling
let hue = use_signal(0);

// In render:
div {
    style: format!("background: hsl({}, 70%, 60%)", hue.get()),
    "Dynamic color"
}
"#
                    }
                }
            }
        }
    }
}
