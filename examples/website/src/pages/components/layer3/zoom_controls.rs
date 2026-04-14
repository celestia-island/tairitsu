use crate::components::breadcrumb;
use tairitsu_macros::rsx;
use tairitsu_vdom::{VElement, VNode, VText};

fn txt(s: &str) -> VNode {
    VNode::Text(VText::new(s))
}
fn el(tag: &str) -> VElement {
    VElement::new(tag)
}

pub fn render() -> VNode {
    let viewport = VNode::Element(el("div").class("zoom-viewport")
        .attr("style","width:100%;height:200px;background:rgba(255,255,255,0.02);border:1px solid rgba(255,255,255,0.08);border-radius:8px;margin-top:16px;display:flex;align-items:center;justify-content:center;color:#555;")
        .child(txt("Zoomable Content Area")));
    rsx! {
        div { id: "page-component-zoom-controls", class: "hikari-page",
            ..vec![breadcrumb(&[("Home", "/"), ("Components", "/components"), ("Layer 3 — Complex", "/components/layer3/zoom-controls"), ("Zoom Controls", "")])]
            section { class: "page-section",
                h2 { class: "page-section__title", "Zoom Controls" }
                p { class: "card__body",
                    "Zoom and pan controls for canvas-based views, maps, diagrams, or large content areas. Supports mouse wheel, pinch gestures, and programmatic control."
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Zoom Toolbar" }
                    div { class: "demo-block__body",
                        div { class: "zoom-toolbar",
                            button { class: "hi-button hi-button-icon", "\u{2B}" }
                            button { class: "hi-button hi-button-icon", "\u{2D}" }
                            span { class: "zoom-level", "100%" }
                            button { class: "hi-button hi-button-icon", "\u{23EB}" }
                            button { class: "hi-button hi-button-icon", "\u{23EC}" }
                            button { class: "hi-button hi-button-icon", "\u{23CE}" }
                        }
                        ..vec![viewport]
                    }
                }
            }
        }
    }
}
