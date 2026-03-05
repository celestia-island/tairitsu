#[allow(dead_code)]
use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

#[allow(dead_code)]
pub fn code_block(code: &str, _language: &str) -> VNode {
    let _code_text = code.to_string();
    rsx! {
        div {
            class: "code-block",
            pre {
                code {
                    "Code placeholder"
                }
            }
        }
    }
}
