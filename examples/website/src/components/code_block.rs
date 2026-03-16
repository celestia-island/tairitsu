#[allow(dead_code)]
use tairitsu_vdom::{Classes, VElement, VNode, VText};

#[allow(dead_code)]
pub fn code_block(code: &str, _language: &str) -> VNode {
    let class_name = format!("code-block language-{}", _language);

    VNode::Element(
        VElement::new("div")
            .class(Classes::new().add(&class_name))
            .child(VNode::Element(VElement::new("pre").child(VNode::Element(
                VElement::new("code").child(VNode::Text(VText::new(code))),
            )))),
    )
}
