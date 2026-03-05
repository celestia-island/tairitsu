use tairitsu_hooks::use_signal;
use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

pub fn rsx_demo() -> VNode {
    let count = use_signal(0);

    rsx! {
        div {
            class: "page rsx-demo",
            h1 {
                "rsx! Macro Demo"
            }

            section {
                class: "demo-section",
                h2 {
                    "1. Basic Elements"
                }
                p {
                    "The rsx! macro provides a declarative way to build UI"
                }
                div {
                    class: "demo-box",
                    button {
                        "Click Me"
                    }
                    input {
                        type: "text",
                        placeholder: "Type here..."
                    }
                }
                pre {
                    code {
                        r#"
rsx! {
    button {
        "Click Me"
    }
    input {
        type: "text",
        placeholder: "Type here..."
    }
}
"#
                    }
                }
            }

            section {
                class: "demo-section",
                h2 {
                    "2. Dynamic Content"
                }
                p {
                    "Use {} to embed dynamic values"
                }
                div {
                    class: "demo-box",
                    p {
                        format!("Count: {}", count.get())
                    }
                    button {
                        onclick: move |_| count.set(count.get() + 1),
                        "Increment"
                    }
                }
                pre {
                    code {
                        r#"
let count = use_signal(0);

rsx! {
    p {
        format!("Count: {}", count.get())
    }
    button {
        onclick: move |_| count.set(count.get() + 1),
        "Increment"
    }
}
"#
                    }
                }
            }

            section {
                class: "demo-section",
                h2 {
                    "3. Attributes and Classes"
                }
                p {
                    "Add attributes and classes to elements"
                }
                div {
                    class: "demo-box",
                    div {
                        class: "styled-box",
                        style: "background: #667eea; padding: 20px; border-radius: 8px;",
                        "Styled with inline attributes"
                    }
                }
                pre {
                    code {
                        r#"
rsx! {
    div {
        class: "styled-box",
        style: "background: #667eea; padding: 20px;",
        "Styled with inline attributes"
    }
}
"#
                    }
                }
            }

            section {
                class: "demo-section",
                h2 {
                    "4. Event Handling"
                }
                p {
                    "Attach event handlers with type-safe callbacks"
                }
                div {
                    class: "demo-box",
                    button {
                        onclick: |_| {
                            web_sys::console::log_1(&"Clicked!".into());
                        },
                        "Click to log"
                    }
                }
                pre {
                    code {
                        r#"
rsx! {
    button {
        onclick: |_| {
                            web_sys::console::log_1(&"Clicked!".into());
        },
        "Click to log"
    }
}
"#
                    }
                }
            }
        }
    }
}
