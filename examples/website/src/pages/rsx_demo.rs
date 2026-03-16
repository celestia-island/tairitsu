#[allow(dead_code)]
use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

use crate::components::code_block::code_block;

#[allow(dead_code)]
pub fn rsx_demo() -> VNode {
    let snippet = r##"rsx! {
    button {
        class: "btn btn-primary",
        onclick: move |_e| {
            tracing::info!("clicked");
        },
        "Click"
    }
}"##;

    rsx! {
        div { class: "page rsx-demo", id: "demo-rsx",
            h3 { "rsx! 宏" }

            section { class: "demo-section", ..vec![code_block(snippet, "rust")],
                p { "声明式语法负责结构组织；事件和属性都在同一棵树里定义。" }
            }
        }
    }
}
