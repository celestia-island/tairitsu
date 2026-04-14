use crate::components::breadcrumb;
use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

pub fn render() -> VNode {
    rsx! {
        div { id: "page-component-qrcode", class: "hikari-page",
            ..vec![breadcrumb(&[("Home", "/"), ("Components", "/components"), ("Layer 2 — Composed", "/components/layer2/qrcode"), ("QRCode", "")])]
            section { class: "page-section",
                h2 { class: "page-section__title", "QRCode" }
                p { class: "card__body",
                    "QR code generation component with customisable size, color, error correction level, and logo embedding."
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Basic QRCode" }
                    div { class: "demo-block__body",
                        div { class: "qrcode-placeholder",
                            style: "width:128px;height:128px;background:rgba(255,255,255,0.05);border-radius:8px;display:flex;align-items:center;justify-content:center;color:#888;font-size:12px;",
                            "QR Code"
                        }
                    }
                }
            }
        }
    }
}
