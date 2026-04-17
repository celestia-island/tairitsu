use crate::components::{breadcrumb, svg_icon};
use hikari_icons::MdiIcon;
use tairitsu_macros::rsx;
use tairitsu_vdom::{VElement, VNode, VText};

fn el(tag: &str) -> VElement {
    VElement::new(tag)
}
fn txt(s: &str) -> VNode {
    VNode::Text(VText::new(s))
}

pub fn render() -> VNode {
    rsx! {
        div { id: "page-component-image", class: "hikari-page",
            ..vec![breadcrumb(&[("Home", "/"), ("Components", "/components"), ("Layer 1 \u{2014} Base", "/components/layer1/image"), ("Image", "")])]
            section { class: "page-section",
                h2 { class: "page-section__title", "Image" }
                p { class: "page-section__description",
                    "Image component with lazy loading, placeholder, object-fit modes, and error fallback."
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Basic Image Placeholder" }
                    div { class: "demo-block__body",
                        div { class: "img-placeholder",
                            style: "width:200px;height:120px;",
                            "Image Placeholder"
                        }
                    }
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Image Sizes" }
                    div { class: "demo-block__body",
                        div { class: "demo-row",
                            div { class: "img-placeholder img-placeholder-sm", "48px" }
                            div { class: "img-placeholder img-placeholder-md", "96px" }
                            div { class: "img-placeholder img-placeholder-lg", "160px" }
                            div { class: "img-placeholder img-placeholder-xl", "240px" }
                        }
                    }
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Image Fit Modes" }
                    div { class: "demo-block__body",
                        div { class: "demo-row",
                            div { class: "img-placeholder-fit", "contain" }
                            div { class: "img-placeholder-fit", "cover" }
                            div { class: "img-placeholder-fit", "fill" }
                        }
                    }
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Error / Loading State" }
                    div { class: "demo-block__body",
                        div { class: "demo-row",
                            div { class: "img-error-state",
                                ..vec![svg_icon(MdiIcon::AlertTriangle, 20, "")]
                                "Failed to load"
                            }
                            div { class: "hi-skeleton", style: "width:160px;height:100px;border-radius:8px;" }
                        }
                    }
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "API" }
                    div { class: "demo-block__body",
                        table { class: "api-table",
                            thead {
                                tr { th { "Property" } th { "Type" } th { "Description" } }
                            }
                            tbody {
                                tr { td { code { "src" } } td { code { "string" } } td { "Image source URL" } }
                                tr { td { code { "alt" } } td { code { "string" } } td { "Alternative text" } }
                                tr { td { code { "fit" } } td { code { "contain | cover | fill | none" } } td { "Object-fit mode" } }
                                tr { td { code { "lazy" } } td { code { "bool" } } td { "Enable lazy loading" } }
                                tr { td { code { "placeholder" } } td { code { "string" } } td { "Placeholder while loading" } }
                                tr { td { code { "fallback" } } td { code { "VNode" } } td { "Error fallback content" } }
                            }
                        }
                    }
                }
            }
        }
    }
}
