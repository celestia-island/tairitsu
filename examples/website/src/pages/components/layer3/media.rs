use crate::components::breadcrumb;
use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

pub fn render() -> VNode {
    rsx! {
        div { id: "page-component-media", class: "hikari-page",
            ..vec![breadcrumb(&[("Home", "/"), ("Components", "/components"), ("Layer 3 — Complex", "/components/layer3/media"), ("Media", "")])]
            section { class: "page-section",
                h2 { class: "page-section__title", "Media" }
                p { class: "card__body",
                    "Rich media player component for audio and video playback with custom controls, playlist support, and streaming."
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Video Player" }
                    div { class: "demo-block__body",
                        div { class: "media-player-placeholder",
                            style: "width:100%;max-width:640px;height:360px;background:rgba(255,255,255,0.03);border-radius:8px;display:flex;align-items:center;justify-content:center;color:#666;flex-direction:column;gap:8px;",
                            "\u{1F3AC}"
                            span { style: "font-size:14px;", "Video Player Component" }
                        }
                    }
                }
            }
        }
    }
}
