#[allow(dead_code)]
use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

#[allow(dead_code)]
pub fn builders() -> VNode {
    rsx! {
        div {
            class: "page builders-demo",
            h1 {
                "Builder System Demo"
            }

            section {
                class: "demo-section",
                h2 {
                    "StyleBuilder"
                }
                p {
                    "Build CSS styles programmatically"
                }
                div {
                    class: "demo-box",
                    style: "background: #667eea; padding: 30px;",
                    h3 {
                        "Dynamic Styling"
                    }
                }
            }
        }
    }
}
