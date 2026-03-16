use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

pub struct App;

impl App {
    pub fn render(&self) -> VNode {
        rsx! {
            div { class: "site-shell",
                header { class: "site-hero", id: "top",
                    div { class: "hero-noise" }
                    p { class: "eyebrow", "Tairitsu Framework Demo" }
                    h1 { "从占位启动到可读 Demo" }
                    p { class: "hero-copy", "这一版先把 docs 里的核心设计翻成可见页面：架构、双后端、WIT 流水线、包分层与开发链路。" }
                    div { class: "hero-actions",
                        a { class: "btn btn-primary", href: "#architecture", "查看系统设计" }
                        a { class: "btn btn-ghost", href: "#commands", "开发命令" }
                    }
                    ul { class: "hero-metrics",
                        li {
                            strong { "3" }
                            span { "层架构" }
                        }
                        li {
                            strong { "2" }
                            span { "Web 后端路径" }
                        }
                        li {
                            strong { "22" }
                            span { "自动生成 WIT 域" }
                        }
                    }
                }

                main { class: "content",
                    section { class: "panel", id: "architecture",
                        h2 { "系统架构总览" }
                        p { class: "lead", "Tairitsu 是面向 WebAssembly Component Model 的运行时，强调接口先行、运行时解耦、双路径并行演进。" }
                        div { class: "grid three",
                            article { class: "card",
                                h3 { "应用层" }
                                p { "自定义 WIT 接口、业务组件、examples 工程，定义你要暴露和消费的能力边界。" }
                            }
                            article { class: "card",
                                h3 { "框架层" }
                                p { "runtime + macros + vdom/hooks/style/web + packager，负责渲染与组件装配。" }
                            }
                            article { class: "card",
                                h3 { "宿主层" }
                                p { "wasmtime/native host 与 browser-glue，承接浏览器能力映射和组件执行。" }
                            }
                        }
                    }

                    section { class: "panel", id: "backends",
                        h2 { "Web 平台双后端" }
                        div { class: "compare",
                            article { class: "card tone-a",
                                h3 { "web 路径" }
                                p { "wasm32-unknown-unknown + wasm-bindgen/web-sys，兼容历史生态。" }
                                ul {
                                    li { "迁移成本低" }
                                    li { "生态成熟" }
                                    li { "协议抽象能力一般" }
                                }
                            }
                            article { class: "card tone-b",
                                h3 { "wit-bindings 路径" }
                                p { "wasm32-wasip2 + browser-glue，直接对齐 Component Model。" }
                                ul {
                                    li { "接口契约更清晰" }
                                    li { "便于未来协议演进" }
                                    li { "需要组件宿主支持" }
                                }
                            }
                        }
                    }

                    section { class: "panel", id: "pipeline",
                        h2 { "W3C WebIDL → WIT 生成流水线" }
                        p { "在脚本链路中抓取 webref IDL，转换为多域 WIT，再合并进 browser-extended 世界。" }
                        ol { class: "steps",
                            li {
                                span { class: "step-index", "01" }
                                div {
                                    h4 { "抓取规范" }
                                    p { "从 w3c/webref 拉取 DOM、Fetch、Streams 等接口定义。" }
                                }
                            }
                            li {
                                span { class: "step-index", "02" }
                                div {
                                    h4 { "生成分域 WIT" }
                                    p { "按 canvas/css/fetch/indexed-db 等域输出 .wit 文件。" }
                                }
                            }
                            li {
                                span { class: "step-index", "03" }
                                div {
                                    h4 { "组装全量世界" }
                                    p { "通过 include 把 Phase 0 与 Phase A 合并为 browser-extended。" }
                                }
                            }
                        }
                    }

                    section { class: "panel", id: "layers",
                        h2 { "包分层职责" }
                        div { class: "grid three",
                            article { class: "card",
                                h3 { "Layer 1 基础运行层" }
                                p { "runtime / macros / vdom" }
                            }
                            article { class: "card",
                                h3 { "Layer 2 平台协议层" }
                                p { "web / browser-worlds / browser-wit-resolver" }
                            }
                            article { class: "card",
                                h3 { "Layer 3 工具交付层" }
                                p { "packager / hooks / style / e2e / browser-glue" }
                            }
                        }
                    }

                    section { class: "panel", id: "commands",
                        h2 { "开发命令速览" }
                        div { class: "command-list",
                            p { class: "cmd", "just dev" }
                            p { class: "cmd", "just build-web" }
                            p { class: "cmd", "just serve-web" }
                            p { class: "cmd", "just wit-gen" }
                        }
                        p { class: "caption", "当前 demo 目标：先保证有可看页面和稳定启动，再继续补交互示例页。" }
                    }
                }

                footer { class: "site-footer",
                    p { "Tairitsu website demo · 2026" }
                    a { href: "#top", "回到顶部" }
                }
            }
        }
    }
}
