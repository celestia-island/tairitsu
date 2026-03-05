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
                }
            }
        }
    }
}
