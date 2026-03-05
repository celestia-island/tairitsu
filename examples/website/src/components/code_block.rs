use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

pub fn code_block(code: &str, language: &str) -> VNode {
    rsx! {
        div {
            class: "code-block",
            pre {
                code {
                    class: format!("language-{}", language),
                    {code.to_string()}
                }
            }
        }
    }
}
