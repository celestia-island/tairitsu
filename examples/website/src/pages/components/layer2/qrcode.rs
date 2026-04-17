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
                            div { class: "qrcode-placeholder qrcode-placeholder--md",
                                "QR Code"
                            }
                        }
                    }
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "QRCode Sizes" }
                    div { class: "demo-block__body",
                         div { class: "demo-row",
                            div { class: "qrcode-placeholder qrcode-placeholder--sm",
                                "64px"
                            }
                            div { class: "qrcode-placeholder qrcode-placeholder--md",
                                "128px"
                            }
                            div { class: "qrcode-placeholder qrcode-placeholder--lg",
                                "192px"
                            }
                        }
                    }
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "QRCode with Content" }
                    div { class: "demo-block__body",
                         div { style: "display:flex;align-items:center;gap:24px;",
                            div { class: "qrcode-placeholder qrcode-placeholder--md",
                                "tairitsu.ai"
                            }
                            div {
                                div { class: "qrcode-info-title", "Tairitsu Website" }
                                div { class: "qrcode-info-desc", "Scan to visit: https://tairitsu.ai" }
                                div { class: "qrcode-info-meta", "Error correction: High" }
                            }
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
                                 tr { td { code { "value" } } td { code { "string" } } td { "QR code content or URL" } }
                                tr { td { code { "size" } } td { code { "number" } } td { "QR code size in pixels (default 128)" } }
                                tr { td { code { "color" } } td { code { "string" } } td { "Foreground color (default #000)" } }
                                tr { td { code { "bgColor" } } td { code { "string" } } td { "Background color (default #fff)" } }
                                tr { td { code { "level" } } td { code { "L | M | Q | H" } } td { "Error correction level (default M)" } }
                                tr { td { code { "logo" } } td { code { "string" } } td { "Embedded logo URL" } }
                            }
                        }
                    }
                }
            }
        }
    }
}
