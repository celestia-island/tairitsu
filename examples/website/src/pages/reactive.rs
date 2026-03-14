#[allow(dead_code)]
use tairitsu_hooks::use_signal;
use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

#[allow(dead_code)]
pub fn reactive() -> VNode {
    let _count = use_signal(0);

    rsx! {
        div { class: "page reactive-demo",
            h1 { "Reactive System Demo" }

            section { class: "demo-section",
                h2 { "use_signal" }
                p { "Signals provide fine-grained reactivity" }
                div { class: "demo-box",
                    p { "Signal state is initialized and ready for interaction" }
                }
            }
        }
    }
}
