//! Home page — mirrors hikari-legacy home page structure.
//!
//! Centered hero with logo, title, subtitle, tagline, and CTA buttons.

use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

pub fn render() -> VNode {
    rsx! {
        div { id: "page-home", class: "hikari-page is-active",
            nav { class: "hi-p-4",
                "Home / Home"
            }
            div { class: "hi-container hi-container-md",
                section { class: "hi-section hi-section-lg",
                    div { class: "hi-section-body",
                        div { class: "hi-text-center",
                            div { class: "hi-mb-4",
                                span { style: "font-size:80px;line-height:1", "\u{273F}" }
                            }
                            h1 { class: "hi-text-2xl hi-text-secondary hi-mb-6",
                                "Tairitsu"
                            }
                            div { style: "height:1.5rem" }
                            p { class: "hi-text-lg hi-text-primary",
                                "A generic WASM Component Runtime Engine."
                            }
                            p { class: "hi-text-sm hi-text-primary",
                                "Docker-like architecture for WASM modules. Type-safe via WIT."
                            }
                        }
                        div { style: "height:2rem" }
                        div { class: "hi-row hi-row-gap-md",
                            style: "display:flex;justify-content:center;gap:1rem;flex-wrap:wrap",
                            a {
                                href: "/guides/quick-start",
                                class: "hi-button hi-button-primary hi-button-lg hi-button-width-auto hi-justify-center",
                                "Quick Start \u{2192}"
                            }
                            a {
                                href: "/system/overview",
                                class: "hi-button hi-button-secondary hi-button-lg hi-button-width-auto hi-justify-center",
                                "System Architecture"
                            }
                        }
                        div { style: "height:3rem" }
                    }
                }
                div { style: "height:2rem" }
                section { class: "hi-section",
                    h2 { class: "hi-text-xl hi-mb-4", "What is Tairitsu?" }
                    div { class: "card-grid",
                        div { class: "card",
                            h3 { class: "card__title", "Image / Container Model" }
                            p { class: "card__body",
                                "Docker-like architecture for managing WASM modules."
                            }
                        }
                        div { class: "card",
                            h3 { class: "card__title", "Generic Runtime" }
                            p { class: "card__body",
                                "No preset WIT interfaces. Pluggable host imports and guest exports."
                            }
                        }
                        div { class: "card",
                            h3 { class: "card__title", "Builder Pattern" }
                            p { class: "card__body",
                                "Flexible Container::builder() API for configuring host imports."
                            }
                        }
                    }
                }
                section { class: "hi-section",
                    h2 { class: "hi-text-xl hi-mb-4", "Architecture Layers" }
                    div { class: "card-grid",
                        div { class: "card",
                            h3 { class: "card__title", "App Layer" }
                            p { class: "card__body",
                                "Custom WIT interfaces, business components, example applications."
                            }
                        }
                        div { class: "card",
                            h3 { class: "card__title", "Framework Layer" }
                            p { class: "card__body",
                                "Runtime + macros + vdom/hooks/style/web + packager."
                            }
                        }
                        div { class: "card",
                            h3 { class: "card__title", "Host Layer" }
                            p { class: "card__body",
                                "wasmtime/native host and browser-glue runtime adaptors."
                            }
                        }
                    }
                }
            }
        }
    }
}
