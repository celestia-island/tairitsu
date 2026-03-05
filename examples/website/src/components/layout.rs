use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

pub fn layout(children: Vec<VNode>) -> VNode {
    rsx! {
        div {
            class: "layout",
            {children}
        }
    }
}
