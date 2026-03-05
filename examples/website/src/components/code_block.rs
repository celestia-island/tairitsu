use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

pub fn code_block(code: &str, _language: &str) -> VNode {
    let code_text = code.to_string();
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
