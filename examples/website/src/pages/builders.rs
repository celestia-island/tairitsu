#[allow(dead_code)]
use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

use crate::components::code_block::code_block;

#[allow(dead_code)]
pub fn builders() -> VNode {
    let snippet = r##"let class = Classes::new()
    .add("panel")
    .add_if("panel-active", is_active);

let style = Style::new()
    .add("padding", "16px")
    .add_custom("--accent", "#c7461f");"##;

    rsx! {
        div { class: "page builders-demo", id: "demo-builders",
            h3 { "Builder 体系" }

            section { class: "demo-section", ..vec![code_block(snippet, "rust")],
                p { "用链式 API 组合 class 与 style，适合做设计系统能力抽象。" }
            }
        }
    }
}
