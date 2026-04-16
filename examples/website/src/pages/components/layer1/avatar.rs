use crate::components::breadcrumb;
use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

pub fn render() -> VNode {
    rsx! {
        div { id: "page-component-avatar", class: "hikari-page",
            ..vec![breadcrumb(&[("Home", "/"), ("Components", "/components"), ("Layer 1 \u{2014} Base", "/components/layer1/avatar"), ("Avatar", "")])]
            section { class: "page-section",
                h2 { class: "page-section__title", "Avatar" }
                p { class: "card__body",
                    "User avatar component with image fallback, size variants, and group display support."
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Avatar Sizes" }
                    div { class: "demo-block__body",
                        div { class: "avatar-row",
                            div { class: "hi-avatar hi-avatar-sm", "S" }
                            div { class: "hi-avatar hi-avatar-md", "M" }
                            div { class: "hi-avatar hi-avatar-lg", "L" }
                            div { class: "hi-avatar hi-avatar-xl", "XL" }
                        }
                    }
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Avatar with Different Colors" }
                    div { class: "demo-block__body",
                        div { class: "avatar-row",
                            div { class: "hi-avatar hi-avatar-lg", style: "background:#e91e63;", "A" }
                            div { class: "hi-avatar hi-avatar-lg", style: "background:#9c27b0;", "B" }
                            div { class: "hi-avatar hi-avatar-lg", style: "background:#2196f3;", "C" }
                            div { class: "hi-avatar hi-avatar-lg", style: "background:#4caf50;", "D" }
                            div { class: "hi-avatar hi-avatar-lg", style: "background:#ff9800;", "E" }
                        }
                    }
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Avatar Group" }
                    div { class: "demo-block__body",
                        div { style: "display:flex;align-items:center;gap:16px;",
                            div { class: "avatar-group",
                                div { class: "hi-avatar hi-avatar-md", "A" }
                                div { class: "hi-avatar hi-avatar-md", "B" }
                                div { class: "hi-avatar hi-avatar-md", "C" }
                                div { class: "hi-avatar hi-avatar-md", style: "background:rgba(255,255,255,0.15);font-size:0.7rem;", "+3" }
                            }
                        }
                    }
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Avatar with Label" }
                    div { class: "demo-block__body",
                        div { style: "display:flex;flex-direction:column;gap:16px;",
                            div { style: "display:flex;align-items:center;gap:12px;",
                                div { class: "hi-avatar hi-avatar-md", "T" }
                                div {
                                    div { style: "font-size:0.875rem;font-weight:500;color:var(--hi-color-text-primary);", "Tairitsu" }
                                    div { style: "font-size:0.8125rem;color:var(--hi-color-text-secondary);", "Framework Author" }
                                }
                            }
                            div { style: "display:flex;align-items:center;gap:12px;",
                                div { class: "hi-avatar hi-avatar-md", style: "background:#e91e63;", "H" }
                                div {
                                    div { style: "font-size:0.875rem;font-weight:500;color:var(--hi-color-text-primary);", "Hikari" }
                                    div { style: "font-size:0.8125rem;color:var(--hi-color-text-secondary);", "UI Designer" }
                                }
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
                                tr { td { code { "size" } } td { code { "sm | md | lg | xl" } } td { "Avatar size" } }
                                tr { td { code { "src" } } td { code { "string" } } td { "Image URL (fallback to initials)" } }
                                tr { td { code { "alt" } } td { code { "string" } } td { "Alt text for image" } }
                                tr { td { code { "shape" } } td { code { "circle | square" } } td { "Avatar shape" } }
                                tr { td { code { "color" } } td { code { "string" } } td { "Custom background color" } }
                            }
                        }
                    }
                }
            }
        }
    }
}
