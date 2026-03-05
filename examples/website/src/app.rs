use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

pub struct App;

impl App {
    pub fn render(&self) -> VNode {
        rsx! {
            div {
                class: "app-container",
                h1 {
                    "Tairitsu Framework Demo"
                }
                p {
                    "Welcome to Tairitsu - A Rust Web Framework"
                }
                div {
                    class: "demo-section",
                    h2 {
                        "rsx! Macro Demo"
                    }
                    p {
                        "This demonstrates the declarative UI syntax"
                    }
                }
                div {
                    class: "demo-section",
                    h2 {
                        "Reactive System"
                    }
                    p {
                        "Fine-grained reactivity with Signals and Effects"
                    }
                }
                div {
                    class: "demo-section",
                    h2 {
                        "Builder System"
                    }
                    p {
                        "Type-safe style and class builders"
                    }
                }
            }
        }
    }
}
