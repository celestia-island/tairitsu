use crate::components::breadcrumb;
use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

pub fn render() -> VNode {
    rsx! {
        div { id: "page-component-image", class: "hikari-page",
            ..vec![breadcrumb(&[("Home", "/"), ("Components", "/components"), ("Layer 1 \u{2014} Base", "/components/layer1/image"), ("Image", "")])]
            section { class: "page-section",
                h2 { class: "page-section__title", "Image" }
                p { class: "card__body",
                    "Image component with lazy loading, placeholder, object-fit modes, and error fallback."
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Basic Image Placeholder" }
                    div { class: "demo-block__body",
                        div { style: "width:200px;height:120px;background:rgba(255,255,255,0.05);border-radius:8px;display:flex;align-items:center;justify-content:center;color:#888;font-size:0.875rem;",
                            "Image Placeholder"
                        }
                    }
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Image Sizes" }
                    div { class: "demo-block__body",
                        div { class: "demo-row",
                            div { style: "width:48px;height:48px;background:rgba(255,255,255,0.05);border-radius:4px;display:flex;align-items:center;justify-content:center;color:#666;font-size:0.625rem;", "48px" }
                            div { style: "width:96px;height:96px;background:rgba(255,255,255,0.05);border-radius:4px;display:flex;align-items:center;justify-content:center;color:#666;font-size:0.75rem;", "96px" }
                            div { style: "width:160px;height:160px;background:rgba(255,255,255,0.05);border-radius:4px;display:flex;align-items:center;justify-content:center;color:#666;font-size:0.875rem;", "160px" }
                            div { style: "width:240px;height:240px;background:rgba(255,255,255,0.05);border-radius:4px;display:flex;align-items:center;justify-content:center;color:#666;font-size:1rem;", "240px" }
                        }
                    }
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Image Fit Modes" }
                    div { class: "demo-block__body",
                        div { class: "demo-row",
                            div { style: "width:120px;height:80px;background:rgba(255,255,255,0.05);border-radius:4px;display:flex;align-items:center;justify-content:center;color:#666;font-size:0.75rem;overflow:hidden;",
                                "contain"
                            }
                            div { style: "width:120px;height:80px;background:rgba(255,255,255,0.05);border-radius:4px;display:flex;align-items:center;justify-content:center;color:#666;font-size:0.75rem;overflow:hidden;",
                                "cover"
                            }
                            div { style: "width:120px;height:80px;background:rgba(255,255,255,0.05);border-radius:4px;display:flex;align-items:center;justify-content:center;color:#666;font-size:0.75rem;overflow:hidden;",
                                "fill"
                            }
                        }
                    }
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Error / Loading State" }
                    div { class: "demo-block__body",
                        div { class: "demo-row",
                            div { style: "width:160px;height:100px;background:rgba(255,255,255,0.03);border:1px dashed var(--hi-color-border);border-radius:8px;display:flex;flex-direction:column;align-items:center;justify-content:center;gap:8px;color:#666;font-size:0.8125rem;",
                                "\u{26A0}"
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
