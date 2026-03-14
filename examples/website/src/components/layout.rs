#[allow(dead_code)]
use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

#[allow(dead_code)]
pub fn layout(_children: Vec<VNode>) -> VNode {
    rsx! {
        div {
            class: "layout",
            // Note: Dynamic children insertion is not yet supported in rsx!
        }
    }
}
