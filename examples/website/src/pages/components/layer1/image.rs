use crate::components::breadcrumb;
use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

pub fn render() -> VNode {
    rsx! {
        div { id: "page-component-image", class: "hikari-page",
            ..vec![breadcrumb(&[("Home", "/"), ("Components", "/components"), ("Layer 1 — Base", "/components/layer1/image"), ("Image", "")])]
            section { class: "page-section",
                h2 { class: "page-section__title", "Image" }
                p { class: "card__body",
                    "Image component with lazy loading, placeholder, object-fit modes, and error fallback."
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Basic Image" }
                    div { class: "demo-block__body",
                        div { class: "image-placeholder",
                            style: "width:200px;height:120px;background:rgba(255,255,255,0.05);border-radius:8px;display:flex;align-items:center;justify-content:center;color:#888;",
                            "Image Placeholder"
                        }
                    }
                }
            }
        }
    }
}
