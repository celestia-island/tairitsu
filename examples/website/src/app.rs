use tairitsu_macros::rsx;
use tairitsu_vdom::{VNode, VText};

use crate::components::code_block::code_block;
use crate::i18n::{text, Locale, LOCALES};

pub struct App;

impl App {
    pub fn render(&self) -> VNode {
        let panes: Vec<VNode> = LOCALES.into_iter().map(render_locale).collect();
        let language_options: Vec<VNode> = LOCALES
            .into_iter()
            .map(|locale| {
                rsx! {
                    option { value: locale.code(), ..txt(locale.label()) }
                }
            })
            .collect();

        rsx! {
            div { class: "site-shell", ..panes,
                div { class: "language-toolbar",
                    label { r#for: "lang-switch", "Language" }
                    select {
                        id: "lang-switch",
                        class: "language-switch",
                        ..language_options,
                    }
                }
            }
        }
    }
}

fn render_locale(locale: Locale) -> VNode {
    let t = text(locale);
    let pane_class = if locale == Locale::ZhChs {
        format!("lang-pane locale-{} is-active", locale.code())
    } else {
        format!("lang-pane locale-{}", locale.code())
    };

    let rsx_code = r##"rsx! {
    button {
        class: "btn btn-primary",
        onclick: move |_e| {
            tracing::info!("clicked");
        },
        "Click"
    }
}"##;

    let builder_code = r##"let class = Classes::new()
    .add("panel")
    .add_if("panel-active", is_active);

let style = Style::new()
    .add("padding", "16px")
    .add_custom("--accent", "#c7461f");"##;

    let quick_start = r##"just dev
just build-web
just serve-web
just wit-gen"##;

    rsx! {
        div { class: pane_class,
            header { class: "site-hero", id: "top",
                div { class: "hero-noise" }
                p { class: "eyebrow", ..txt(t.brand) }
                h1 { ..txt(t.hero_title) }
                p { class: "hero-copy", ..txt(t.hero_copy) }
                div { class: "hero-actions",
                    a {
                        class: "btn btn-primary",
                        href: "#architecture",
                        ..txt(t.action_primary),
                    }
                    a {
                        class: "btn btn-ghost",
                        href: "#commands",
                        ..txt(t.action_secondary),
                    }
                }
                ul { class: "hero-metrics",
                    li {
                        strong { "3" }
                        span { "Layers" }
                    }
                    li {
                        strong { "2" }
                        span { "Backends" }
                    }
                    li {
                        strong { "22" }
                        span { "WIT domains" }
                    }
                }
            }

            nav { class: "section-nav",
                ul {
                    li {
                        a { href: "#architecture", ..txt(t.nav_arch) }
                    }
                    li {
                        a { href: "#backends", ..txt(t.nav_backend) }
                    }
                    li {
                        a { href: "#pipeline", ..txt(t.nav_pipeline) }
                    }
                    li {
                        a { href: "#demos", ..txt(t.nav_demo) }
                    }
                    li {
                        a { href: "#commands", ..txt(t.nav_cmd) }
                    }
                }
            }

            main { class: "content",
                section { class: "panel", id: "architecture",
                    h2 { ..txt(t.section_arch) }
                    p { class: "lead", ..txt(t.section_arch_lead) }
                    div { class: "grid three",
                        article { class: "card",
                            h3 { "App Layer" }
                            p { "Custom WIT interfaces, business components, example apps." }
                        }
                        article { class: "card",
                            h3 { "Framework Layer" }
                            p { "runtime + macros + vdom/hooks/style/web + packager." }
                        }
                        article { class: "card",
                            h3 { "Host Layer" }
                            p { "wasmtime/native host and browser-glue runtime adaptors." }
                        }
                    }
                }

                section { class: "panel", id: "backends",
                    h2 { ..txt(t.section_backend) }
                    div { class: "compare",
                        article { class: "card tone-a",
                            h3 { "web" }
                            p { "wasm32-unknown-unknown + wasm-bindgen/web-sys" }
                        }
                        article { class: "card tone-b",
                            h3 { "wit-bindings" }
                            p { "wasm32-wasip2 + browser-glue + component host" }
                        }
                    }
                }

                section { class: "panel", id: "pipeline",
                    h2 { ..txt(t.section_pipeline) }
                    ol { class: "steps",
                        li {
                            span { class: "step-index", "01" }
                            div {
                                h4 { "Fetch specs" }
                                p { "Pull IDL from webref." }
                            }
                        }
                        li {
                            span { class: "step-index", "02" }
                            div {
                                h4 { "Generate WIT" }
                                p { "Convert into domain WIT files." }
                            }
                        }
                        li {
                            span { class: "step-index", "03" }
                            div {
                                h4 { "Compose world" }
                                p { "Include phase-0 plus generated domains." }
                            }
                        }
                    }
                }

                section { class: "panel", id: "demos",
                    h2 { ..txt(t.section_demo) }
                    div { class: "demo-grid",
                        div {
                            class: "page",
                            ..vec![code_block(quick_start, "bash")],
                            h3 { "Quick Start" }
                        }
                        div { class: "page", ..vec![code_block(rsx_code, "rust")],
                            h3 { "rsx!" }
                        }
                        div {
                            class: "page",
                            ..vec![code_block(builder_code, "rust")],
                            h3 { "Builder" }
                        }
                        div { class: "page",
                            h3 { "Reactive" }
                            p {
                                "Signal and Effect API are available and ready for interactive expansion."
                            }
                        }
                    }
                }

                section { class: "panel", id: "commands",
                    h2 { ..txt(t.section_cmd) }
                    div { class: "command-list",
                        p { class: "cmd", "just dev" }
                        p { class: "cmd", "just build-web" }
                        p { class: "cmd", "just serve-web" }
                        p { class: "cmd", "just wit-gen" }
                    }
                    p { class: "caption", ..txt(t.cmd_caption) }
                }
            }

            footer { class: "site-footer",
                p { "Tairitsu website demo · 2026" }
                a { href: "#top", "Top" }
            }
        }
    }
}

fn txt(value: &str) -> Vec<VNode> {
    vec![VNode::Text(VText::new(value))]
}
