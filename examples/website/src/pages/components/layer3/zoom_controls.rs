use crate::components::breadcrumb;
use tairitsu_macros::rsx;
use tairitsu_vdom::{VElement, VNode, VText};

fn el(tag: &str) -> VElement {
    VElement::new(tag)
}
fn txt(s: &str) -> VNode {
    VNode::Text(VText::new(s))
}

pub fn render() -> VNode {
    let viewport = VNode::Element(el("div").class("zoom-viewport")
        .attr("style","width:100%;height:200px;background:rgba(255,255,255,0.02);border:1px solid var(--hi-color-border);border-radius:8px;margin-top:16px;display:flex;align-items:center;justify-content:center;color:#555;")
        .child(txt("Zoomable Content Area")));

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
                            button { class: "hi-button", style: "padding:4px 8px;font-size:1rem;", "\u{2B}" }
                            button { class: "hi-button", style: "padding:4px 8px;font-size:1rem;", "\u{2D}" }
                            span { class: "zoom-level", "100%" }
                            button { class: "hi-button", style: "padding:4px 8px;", "\u{23EB}" }
                            button { class: "hi-button", style: "padding:4px 8px;", "\u{23EC}" }
                            button { class: "hi-button", style: "padding:4px 8px;", "\u{23CE}" }
                        }
                        ..vec![viewport]
                    }
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Floating Zoom Controls" }
                    div { class: "demo-block__body",
                        div { style: "position:relative;width:100%;height:240px;background:rgba(255,255,255,0.02);border:1px solid var(--hi-color-border);border-radius:8px;overflow:hidden;",
                            div { style: "position:absolute;bottom:16px;right:16px;display:flex;flex-direction:column;gap:4px;background:var(--hi-color-surface);border:1px solid var(--hi-color-border);border-radius:8px;padding:4px;",
                                button { class: "hi-button", style: "padding:4px 8px;font-size:0.875rem;", "+" }
                                button { class: "hi-button", style: "padding:4px 8px;font-size:0.875rem;", "\u{2212}" }
                                div { style: "height:1px;background:var(--hi-color-border);margin:2px 0;" }
                                button { class: "hi-button", style: "padding:4px 8px;font-size:0.875rem;", "\u{23CE}" }
                            }
                            div { style: "position:absolute;bottom:16px;left:16px;display:flex;align-items:center;gap:8px;background:var(--hi-color-surface);border:1px solid var(--hi-color-border);border-radius:8px;padding:6px 12px;",
                                span { style: "font-size:0.8125rem;color:var(--hi-color-text-secondary);font-family:var(--ts-font-mono);", "Zoom: 100%" }
                            }
                            div { style: "position:absolute;top:50%;left:50%;transform:translate(-50%,-50%);color:var(--hi-color-text-disabled);font-size:0.875rem;",
                                "Scroll to zoom, drag to pan"
                            }
                        }
                    }
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Zoom Slider" }
                    div { class: "demo-block__body",
                        div { style: "display:flex;align-items:center;gap:12px;max-width:300px;",
                            span { style: "font-size:0.8125rem;color:var(--hi-color-text-disabled);", "25%" }
                            div { style: "flex:1;height:4px;background:rgba(255,255,255,0.08);border-radius:2px;position:relative;",
                                div { style: "width:40%;height:100%;background:var(--ts-color-primary);border-radius:2px;" }
                                div { style: "position:absolute;left:40%;top:50%;transform:translate(-50%,-50%);width:14px;height:14px;border-radius:50%;background:#fff;border:2px solid var(--ts-color-primary);" }
                            }
                            span { style: "font-size:0.8125rem;color:var(--hi-color-text-disabled);", "400%" }
                        }
                    }
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "API" }
                    div { class: "demo-block__body",
                        table { class: "api-table",
                            thead {
                                tr { th { "Property" } th { "Type" } th { "Default" } th { "Description" } }
                            }
                            tbody {
                                tr { td { code { "min" } } td { code { "number" } } td { code { "0.1" } } td { "Minimum zoom level" } }
                                tr { td { code { "max" } } td { code { "number" } } td { code { "10" } } td { "Maximum zoom level" } }
                                tr { td { code { "step" } } td { code { "number" } } td { code { "0.1" } } td { "Zoom step size" } }
                                tr { td { code { "value" } } td { code { "number" } } td { code { "1" } } td { "Current zoom level" } }
                                tr { td { code { "pannable" } } td { code { "bool" } } td { code { "true" } } td { "Enable drag-to-pan" } }
                                tr { td { code { "wheelZoom" } } td { code { "bool" } } td { code { "true" } } td { "Enable mouse wheel zoom" } }
                                tr { td { code { "onZoomChange" } } td { code { "(zoom: number) => void" } } td { "-" } td { "Zoom level change callback" } }
                            }
                        }
                    }
                }
            }
        }
    }
}
