use crate::components::breadcrumb;
use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

pub fn render() -> VNode {
    rsx! {
        div { id: "page-component-media", class: "hikari-page",
            ..vec![breadcrumb(&[("Home", "/"), ("Components", "/components"), ("Layer 3 \u{2014} Complex", "/components/layer3/media"), ("Media", "")])]
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
                            span { style: "font-size:0.875rem;", "Video Player Component" }
                            span { style: "font-size:0.75rem;color:var(--hi-color-text-disabled);", "16:9 aspect ratio" }
                        }
                    }
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Video Controls" }
                    div { class: "demo-block__body",
                        div { style: "max-width:640px;",
                            div { style: "display:flex;align-items:center;gap:12px;padding:8px 0;border-bottom:1px solid var(--hi-color-border);",
                                button { class: "hi-button", style: "padding:4px 8px;font-size:1rem;", "\u{25B6}" }
                                div { style: "flex:1;height:4px;background:rgba(255,255,255,0.08);border-radius:2px;overflow:hidden;",
                                    div { style: "width:35%;height:100%;background:var(--ts-color-primary);border-radius:2px;" }
                                }
                                span { style: "font-size:0.75rem;color:var(--hi-color-text-disabled);font-family:var(--ts-font-mono);", "1:24 / 4:02" }
                                button { class: "hi-button", style: "padding:4px 8px;font-size:0.875rem;", "\u{1F50A}" }
                                button { class: "hi-button", style: "padding:4px 8px;font-size:0.875rem;", "\u{26F6}" }
                            }
                        }
                    }
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Audio Player" }
                    div { class: "demo-block__body",
                        div { style: "max-width:480px;background:rgba(255,255,255,0.03);border:1px solid var(--hi-color-border);border-radius:12px;padding:16px;",
                            div { style: "display:flex;align-items:center;gap:12px;margin-bottom:12px;",
                                div { style: "width:48px;height:48px;background:var(--ts-color-primary);border-radius:8px;display:flex;align-items:center;justify-content:center;font-size:1.25rem;", "\u{266B}" }
                                div {
                                    div { style: "font-size:0.9375rem;font-weight:500;color:var(--hi-color-text-primary);", "WASM Symphony" }
                                    div { style: "font-size:0.8125rem;color:var(--hi-color-text-secondary);", "Tairitsu Orchestra" }
                                }
                            }
                            div { style: "margin-bottom:8px;",
                                div { style: "height:4px;background:rgba(255,255,255,0.08);border-radius:2px;overflow:hidden;",
                                    div { style: "width:60%;height:100%;background:var(--ts-color-primary);border-radius:2px;" }
                                }
                                div { style: "display:flex;justify-content:space-between;margin-top:4px;",
                                    span { style: "font-size:0.6875rem;color:var(--hi-color-text-disabled);font-family:var(--ts-font-mono);", "2:15" }
                                    span { style: "font-size:0.6875rem;color:var(--hi-color-text-disabled);font-family:var(--ts-font-mono);", "3:45" }
                                }
                            }
                            div { style: "display:flex;align-items:center;justify-content:center;gap:24px;",
                                button { class: "hi-button", style: "padding:4px 8px;", "\u{23EE}" }
                                button { class: "hi-button hi-button-primary", style: "width:40px;height:40px;border-radius:50%;padding:0;display:flex;align-items:center;justify-content:center;", "\u{25B6}" }
                                button { class: "hi-button", style: "padding:4px 8px;", "\u{23ED}" }
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
                                tr { td { code { "src" } } td { code { "string" } } td { "Media source URL" } }
                                tr { td { code { "type" } } td { code { "video | audio" } } td { "Media type" } }
                                tr { td { code { "autoplay" } } td { code { "bool" } } td { "Auto-play on mount" } }
                                tr { td { code { "controls" } } td { code { "bool" } } td { "Show native controls" } }
                                tr { td { code { "loop" } } td { code { "bool" } } td { "Loop playback" } }
                                tr { td { code { "playlist" } } td { code { "MediaItem[]" } } td { "Playlist items" } }
                                tr { td { code { "onPlay" } } td { code { "() => void" } } td { "Play event callback" } }
                            }
                        }
                    }
                }
            }
        }
    }
}
