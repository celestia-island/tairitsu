#[allow(dead_code)]
use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

#[allow(dead_code)]
pub fn code_block(code: &str, _language: &str) -> VNode {
    let _ = code;
    rsx! {
        div { class: "code-block",
            pre {
                code { "Code example is rendered by the current website demo skin" }
            }
        }
    }
}
