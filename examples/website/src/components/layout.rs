use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

pub fn layout(children: Vec<VNode>) -> VNode {
    rsx! {
        div {
            class: "layout",
            // Note: Dynamic children insertion not yet supported in rsx!
            // This is a placeholder for future enhancement
        }
    }
}
