use crate::components::navigation::navigation;
use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

pub fn home() -> VNode {
    rsx! {
        div {
            class: "page home",
            {navigation()},
            header {
                class: "hero",
                h1 {
                    "Tairitsu Framework"
                }
                p {
                    class: "tagline",
                    "A modern Rust Web Framework with Virtual DOM, Reactive System, and Type-Safe UI"
                }
            }
            section {
                class: "features",
                h2 {
                    "Core Features"
                }
                div {
                    class: "feature-grid",
                    div {
                        class: "feature-card",
                        h3 {
                            "rsx! Macro"
                        }
                        p {
                            "Declarative UI syntax with compile-time safety"
                        }
                    }
                    div {
                        class: "feature-card",
                        h3 {
                            "Reactive System"
                        }
                        p {
                            "Fine-grained reactivity with Signals and Effects"
                        }
                    }
                    div {
                        class: "feature-card",
                        h3 {
                            "Builder System"
                        }
                        p {
                            "Type-safe style and class builders"
                        }
                    }
                    div {
                        class: "feature-card",
                        h3 {
                            "Platform Abstraction"
                        }
                        p {
                            "Cross-platform WebPlatform implementation"
                        }
                    }
                }
            }
            section {
                class: "quick-start",
                h2 {
                    "Quick Start"
                }
                pre {
                    code {
                        r#"
// 1. Add dependencies
[dependencies]
tairitsu-vdom = "0.1"
tairitsu-hooks = "0.1"
tairitsu-macros = "0.1"

// 2. Create your app
use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

fn app() -> VNode {
    rsx! {
        div {
            h1 { "Hello, Tairitsu!" }
        }
    }
}
"#
                    }
                }
            }
        }
    }
}
