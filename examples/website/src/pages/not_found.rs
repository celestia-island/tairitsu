//! 404 Not Found page

use tairitsu_macros::rsx;
use tairitsu_vdom::{VElement, VNode, VText};

use crate::i18n::{self, Language};

fn txt(s: &str) -> VNode {
    VNode::Text(VText::new(s))
}

fn el(tag: &str) -> VElement {
    VElement::new(tag)
}

pub fn render() -> VNode {
    let t = i18n::text(Language::ENGLISH);
    rsx! {
        div { id: "page-not-found", class: "hikari-page",
            div { class: "hi-container hi-container-md",
                section { class: "hi-section hi-section-lg",
                    div { class: "hi-section-body",
                        div { class: "hi-text-center",
                            h1 { class: "hi-text-2xl hi-text-secondary hi-mb-6", "404" }
                            ..vec![
                                VNode::Element(el("p").class("hi-text-lg hi-text-primary").child(txt(t.not_found_title))),
                                VNode::Element(el("p").class("hi-text-sm hi-text-primary").child(txt(t.not_found_desc))),
                                VNode::Element(el("div").attr("style", "height:2rem")),
                                VNode::Element(
                                    el("a")
                                        .attr("href", "/")
                                        .class("hi-button hi-button-primary hi-button-md hi-button-width-auto hi-justify-center")
                                        .child(txt(t.not_found_action)),
                                ),
                            ]
                        }
                    }
                }
            }
        }
    }
}
