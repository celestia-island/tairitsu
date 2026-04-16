use crate::components::breadcrumb;
use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

pub fn render() -> VNode {
    rsx! {
        div { id: "page-component-qrcode", class: "hikari-page",
            ..vec![breadcrumb(&[("Home", "/"), ("Components", "/components"), ("Layer 2 \u{2014} Composed", "/components/layer2/qrcode"), ("QRCode", "")])]
            section { class: "page-section",
                h2 { class: "page-section__title", "QRCode" }
                p { class: "card__body",
                    "QR code generation component with customisable size, color, error correction level, and logo embedding."
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Basic QRCode" }
                    div { class: "demo-block__body",
                        div { class: "demo-row",
                            div { class: "qrcode-placeholder",
                                style: "width:128px;height:128px;background:rgba(255,255,255,0.05);border-radius:8px;display:flex;align-items:center;justify-content:center;color:#888;font-size:0.8125rem;",
                                "QR Code"
                            }
                        }
                    }
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "QRCode Sizes" }
                    div { class: "demo-block__body",
                        div { class: "demo-row",
                            div { class: "qrcode-placeholder",
                                style: "width:64px;height:64px;background:rgba(255,255,255,0.05);border-radius:4px;display:flex;align-items:center;justify-content:center;color:#888;font-size:0.625rem;",
                                "64px"
                            }
                            div { class: "qrcode-placeholder",
                                style: "width:128px;height:128px;background:rgba(255,255,255,0.05);border-radius:8px;display:flex;align-items:center;justify-content:center;color:#888;font-size:0.8125rem;",
                                "128px"
                            }
                            div { class: "qrcode-placeholder",
                                style: "width:192px;height:192px;background:rgba(255,255,255,0.05);border-radius:12px;display:flex;align-items:center;justify-content:center;color:#888;font-size:1rem;",
                                "192px"
                            }
                        }
                    }
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "QRCode with Content" }
                    div { class: "demo-block__body",
                        div { style: "display:flex;align-items:center;gap:24px;",
                            div { class: "qrcode-placeholder",
                                style: "width:128px;height:128px;background:rgba(255,255,255,0.05);border-radius:8px;display:flex;align-items:center;justify-content:center;color:#888;font-size:0.75rem;",
                                "tairitsu.ai"
                            }
                            div {
                                div { style: "font-size:0.875rem;color:var(--hi-color-text-primary);font-weight:500;margin-bottom:4px;", "Tairitsu Website" }
                                div { style: "font-size:0.8125rem;color:var(--hi-color-text-secondary);margin-bottom:4px;", "Scan to visit: https://tairitsu.ai" }
                                div { style: "font-size:0.75rem;color:var(--hi-color-text-disabled);", "Error correction: High" }
                            }
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
                                tr { td { code { "value" } } td { code { "string" } } td { "-" } td { "QR code content/URL" } }
                                tr { td { code { "size" } } td { code { "number" } } td { code { "128" } } td { "QR code size in pixels" } }
                                tr { td { code { "color" } } td { code { "string" } } td { code { "#000" } } td { "Foreground color" } }
                                tr { td { code { "bgColor" } } td { code { "string" } } td { code { "#fff" } } td { "Background color" } }
                                tr { td { code { "level" } } td { code { "L | M | Q | H" } } td { code { "M" } } td { "Error correction level" } }
                                tr { td { code { "logo" } } td { code { "string" } } td { "-" } td { "Embedded logo URL" } }
                            }
                        }
                    }
                }
            }
        }
    }
}
