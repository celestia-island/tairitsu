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
    let viewport = VNode::Element(
        el("div")
            .class("zoom-viewport")
            .child(txt("Zoomable Content Area")),
    );

    rsx! {
        div { id: "page-component-zoom-controls", class: "hikari-page",
            ..vec![breadcrumb(&[("Home", "/"), ("Components", "/components"), ("Layer 3 \u{2014} Complex", "/components/layer3/zoom-controls"), ("Zoom Controls", "")])]
            section { class: "page-section",
                h2 { class: "page-section__title", "Zoom Controls" }
                p { class: "card__body",
                    "Zoom and pan controls for canvas-based views, maps, diagrams, or large content areas. Supports mouse wheel, pinch gestures, and programmatic control."
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Zoom Toolbar" }
                    div { class: "demo-block__body",
                        div { class: "zoom-toolbar",
                             button { class: "hi-button zoom-toolbar-btn", ..vec![svg_icon(MdiIcon::Plus, 14, "")] }
                             button { class: "hi-button zoom-toolbar-btn", ..vec![svg_icon(MdiIcon::Minus, 14, "")] }
                             span { class: "zoom-level", "100%" }
                             button { class: "hi-button zoom-toolbar-btn zoom-toolbar-btn--md", ..vec![svg_icon(MdiIcon::ChevronUp, 14, "")] }
                             button { class: "hi-button zoom-toolbar-btn zoom-toolbar-btn--md", ..vec![svg_icon(MdiIcon::ChevronDown, 14, "")] }
                             button { class: "hi-button zoom-toolbar-btn zoom-toolbar-btn--md", ..vec![svg_icon(MdiIcon::Maximize2, 14, "")] }
                        }
                        ..vec![viewport]
                    }
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Floating Zoom Controls" }
                    div { class: "demo-block__body",
                         div { style: "position:relative;width:100%;height:240px;background:rgba(255,255,255,0.02);border:1px solid var(--hi-color-border);border-radius:8px;overflow:hidden;",
                            div { class: "zoom-float-panel",
                                button { class: "hi-button zoom-toolbar-btn zoom-toolbar-btn--md", ..vec![svg_icon(MdiIcon::Plus, 14, "")] }
                                button { class: "hi-button zoom-toolbar-btn zoom-toolbar-btn--md", ..vec![svg_icon(MdiIcon::Minus, 14, "")] }
                                div { class: "zoom-float-divider" }
                                button { class: "hi-button zoom-toolbar-btn zoom-toolbar-btn--md", ..vec![svg_icon(MdiIcon::Maximize2, 14, "")] }
                            }
                            div { class: "zoom-float-badge",
                                span { class: "zoom-float-badge__text", "Zoom: 100%" }
                            }
                            div { class: "zoom-float-hint",
                                "Scroll to zoom, drag to pan"
                            }
                        }
                    }
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Zoom Slider" }
                    div { class: "demo-block__body",
                         div { class: "zoom-slider-wrap",
                            span { class: "zoom-slider-label", "25%" }
                            div { class: "zoom-slider-track",
                                div { class: "zoom-slider-fill", style: "width:40%;" }
                                div { class: "zoom-slider-thumb" }
                            }
                            span { class: "zoom-slider-label", "400%" }
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
                                 tr { td { code { "min" } } td { code { "number" } } td { "Minimum zoom level (default 0.1)" } }
                                tr { td { code { "max" } } td { code { "number" } } td { "Maximum zoom level (default 10)" } }
                                tr { td { code { "step" } } td { code { "number" } } td { "Zoom step size (default 0.1)" } }
                                tr { td { code { "value" } } td { code { "number" } } td { "Current zoom level (default 1)" } }
                                tr { td { code { "pannable" } } td { code { "bool" } } td { "Enable drag-to-pan (default true)" } }
                                tr { td { code { "wheelZoom" } } td { code { "bool" } } td { "Enable mouse wheel zoom (default true)" } }
                                tr { td { code { "onZoomChange" } } td { code { "(zoom: number) => void" } } td { "Zoom level change callback" } }
                            }
                        }
                    }
                }
            }
        }
    }
}
