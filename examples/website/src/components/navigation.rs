#[allow(dead_code)]
use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

#[allow(dead_code)]
pub fn navigation() -> VNode {
    rsx! {
        nav { class: "section-nav",
            ul {
                li {
                    a { href: "#architecture", "架构" }
                }
                li {
                    a { href: "#backends", "双后端" }
                }
                li {
                    a { href: "#pipeline", "WIT 流水线" }
                }
                li {
                    a { href: "#demos", "机制演示" }
                }
                li {
                    a { href: "#commands", "命令" }
                }
            }
        }
    }
}
