use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

pub fn home() -> VNode {
    rsx! {
        div {
            class: "page home",
            header {
                class: "hero",
                h1 {
                    "Tairitsu Framework"
                }
                p {
                    class: "tagline",
                    "A modern Rust Web Framework"
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
                            "Declarative UI syntax"
                        }
                    }
                }
            }
        }
    }
}
