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

    let workspace_map = r##"packages/     Rust 核心包与工具包
examples/     示例工程（不同 WIT 集成路径）
scripts/      WebIDL/WIT 生成与辅助脚本
docs/         项目文档（多语言）
tests/        端到端相关资产"##;

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
                        a { href: "#runtime", "Runtime" }
                    }
                    li {
                        a { href: "#workspace", "Workspace" }
                    }
                    li {
                        a { href: "#packages", "Packages" }
                    }
                    li {
                        a { href: "#examples", "Examples" }
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
                            h3 { "component" }
                            p { "wasm32-wasip2 + WIT Component Model" }
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

                section { class: "panel", id: "runtime",
                    h2 { ..txt(t.section_runtime) }
                    p { class: "lead", ..txt(t.section_runtime_lead) }
                    div { class: "flow-grid",
                        article { class: "flow-card",
                            span { class: "kicker", "01" }
                            h3 { "Image" }
                            p { "Immutable WASM component binary plus metadata." }
                        }
                        article { class: "flow-card",
                            span { class: "kicker", "02" }
                            h3 { "Container::builder(image)" }
                            p {
                                "Configure host imports, guest initializer, linker state, and execution context."
                            }
                        }
                        article { class: "flow-card",
                            span { class: "kicker", "03" }
                            h3 { "Container" }
                            p {
                                "Running instance that encapsulates store, linker, and invocation context."
                            }
                        }
                        article { class: "flow-card",
                            span { class: "kicker", "04" }
                            h3 { "Invoke" }
                            p {
                                "Choose typed bindings for fixed contracts or dynamic ABI / RON for plugin-style access."
                            }
                        }
                    }
                    div { class: "badge-row",
                        span { class: "badge", "typed bindings" }
                        span { class: "badge", "dynamic invocation" }
                        span { class: "badge", "guest initializer" }
                        span { class: "badge", "host linker" }
                    }
                }

                section { class: "panel", id: "workspace",
                    h2 { ..txt(t.section_workspace) }
                    p { class: "lead", ..txt(t.section_workspace_lead) }
                    div { class: "workspace-layout",
                        div {
                            class: "repo-map",
                            ..vec![code_block(workspace_map, "text")],
                        }
                        div { class: "repo-notes",
                            article { class: "mini-card",
                                h3 { "browser-worlds" }
                                p { "Defines browser WIT worlds and generated domain packages." }
                            }
                            article { class: "mini-card",
                                h3 { "runtime" }
                                p { "Carries image/container abstractions and component execution." }
                            }
                            article { class: "mini-card",
                                h3 { "web" }
                                p { "Provides component-first browser platform bindings." }
                            }
                            article { class: "mini-card",
                                h3 { "packager" }
                                p { "Builds wasm or component targets and emits browser host assets." }
                            }
                        }
                    }
                }

                section { class: "panel", id: "packages",
                    h2 { ..txt(t.section_packages) }
                    div { class: "package-grid",
                        article { class: "package-card",
                            h3 { "packages/runtime" }
                            p { "Image, Container, builder API, Wasmtime component execution." }
                        }
                        article { class: "package-card",
                            h3 { "packages/macros" }
                            p { "rsx!, wit_world!, and macro helpers used by examples and apps." }
                        }
                        article { class: "package-card",
                            h3 { "packages/vdom" }
                            p { "Platform-agnostic VDOM nodes, styles, classes, and event model." }
                        }
                        article { class: "package-card",
                            h3 { "packages/web" }
                            p { "WebPlatform and WitPlatform implementations for two browser paths." }
                        }
                        article { class: "package-card",
                            h3 { "packages/browser-wit-resolver" }
                            p { "WIT resolution, local cache, fetch pipeline, and registry helpers." }
                        }
                        article { class: "package-card",
                            h3 { "packages/packager" }
                            p {
                                "CLI build orchestration, dev server, asset copy, and component host HTML."
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

                section { class: "panel", id: "examples",
                    h2 { ..txt(t.section_examples) }
                    ol { class: "example-order",
                        li {
                            strong { "01" }
                            div {
                                h4 { "wit-native-macro" }
                                p {
                                    "Macro-assisted entry point for understanding generated WIT integration."
                                }
                            }
                        }
                        li {
                            strong { "02" }
                            div {
                                h4 { "wit-native-simple" }
                                p { "Simple trait implementation and direct host-guest shape." }
                            }
                        }
                        li {
                            strong { "03" }
                            div {
                                h4 { "wit-runtime" }
                                p { "Runtime loading model and component execution path." }
                            }
                        }
                        li {
                            strong { "04" }
                            div {
                                h4 { "wit-compile-time" }
                                p { "Compile-time binding flow and WIT parsing path." }
                            }
                        }
                        li {
                            strong { "05" }
                            div {
                                h4 { "wit-dynamic-advanced" }
                                p {
                                    "Dynamic invocation, RON serialization, and discovery-oriented flow."
                                }
                            }
                        }
                        li {
                            strong { "06" }
                            div {
                                h4 { "website" }
                                p {
                                    "Browser-facing docs demo that stitches together VDOM, styles, hooks, and packaging."
                                }
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
