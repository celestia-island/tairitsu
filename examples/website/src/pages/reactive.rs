#[allow(dead_code)]
use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

use crate::components::code_block::code_block;

#[allow(dead_code)]
pub fn reactive() -> VNode {
    let snippet = r##"let count = use_signal(0);

use_effect({
    let count = count.clone();
    move || tracing::info!("count changed: {}", count.get())
});

count.set(count.get() + 1);"##;

    rsx! {
        div { class: "page reactive-demo", id: "demo-reactive",
            h3 { "响应式基础" }

            section { class: "demo-section",
                p { "Signal / Effect 已具备表达能力，下一步可继续接入自动重渲染驱动。" }
                ..vec![code_block(snippet, "rust")]
            }
        }
    }
}
