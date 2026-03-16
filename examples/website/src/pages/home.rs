#[allow(dead_code)]
use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

use crate::components::code_block::code_block;

#[allow(dead_code)]
pub fn home() -> VNode {
    let quick_start = r#"# in workspace root
just dev

# production build
just build-web
just serve-web"#;

    rsx! {
        div { class: "page home", id: "demo-home",
            h3 { "快速开始" }
            p {
                "把文档中的命令先跑通，确认 component 构建、资源复制、dev watch 全链路可用。"
            }
            section { class: "demo-section", ..vec![code_block(quick_start, "bash")] }
        }
    }
}
