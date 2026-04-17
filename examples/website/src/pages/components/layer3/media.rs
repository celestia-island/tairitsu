use crate::components::{breadcrumb, svg_icon};
use hikari_icons::MdiIcon;
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
                            ..vec![svg_icon(MdiIcon::Image, 48, "media-player-placeholder__meta")]
                            span { class: "media-player-placeholder__label", "Video Player Component" }
                            span { class: "media-player-placeholder__meta", "16:9 aspect ratio" }
                        }
                    }
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Video Controls" }
                    div { class: "demo-block__body",
                         div { style: "max-width:640px;",
                            div { class: "media-controls-bar",
                                button { class: "hi-button media-control-btn", ..vec![svg_icon(MdiIcon::Play, 16, "")] }
                                div { class: "media-progress-track",
                                    div { class: "media-progress-fill", style: "width:35%;" }
                                }
                                span { class: "media-time-display", "1:24 / 4:02" }
                                button { class: "hi-button media-control-btn", ..vec![svg_icon(MdiIcon::VolumeHigh, 16, "")] }
                                button { class: "hi-button media-control-btn", ..vec![svg_icon(MdiIcon::Fullscreen, 16, "")] }
                            }
                        }
                    }
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Audio Player" }
                    div { class: "demo-block__body",
                         div { class: "audio-player-card",
                            div { class: "audio-player-header",
                                div { class: "audio-player-artwork", ..vec![svg_icon(MdiIcon::Music, 24, "")] }
                                div {
                                    div { class: "audio-player-info-title", "WASM Symphony" }
                                    div { class: "audio-player-info-subtitle", "Tairitsu Orchestra" }
                                }
                            }
                            div { style: "margin-bottom:8px;",
                                div { class: "media-progress-track",
                                    div { class: "media-progress-fill", style: "width:60%;" }
                                }
                                div { style: "display:flex;justify-content:space-between;margin-top:4px;",
                                    span { class: "audio-player-time", "2:15" }
                                    span { class: "audio-player-time", "3:45" }
                                }
                            }
                            div { class: "audio-player-controls",
                                button { class: "hi-button media-control-btn", ..vec![svg_icon(MdiIcon::Stop, 16, "")] }
                                button { class: "hi-button hi-button-primary audio-play-btn", ..vec![svg_icon(MdiIcon::Play, 18, "")] }
                                button { class: "hi-button media-control-btn", ..vec![svg_icon(MdiIcon::Pause, 16, "")] }
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
