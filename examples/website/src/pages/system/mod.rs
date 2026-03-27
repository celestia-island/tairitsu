//! System pages - architecture and runtime documentation

pub mod overview;
pub mod runtime;
pub mod versioning;
pub mod web_backends;
pub mod wit_pipeline;

use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

/// Render all system pages
pub fn render_all() -> Vec<VNode> {
    vec![
        render_overview(),
        overview::render(),
        runtime::render(),
        wit_pipeline::render(),
        web_backends::render(),
        versioning::render(),
    ]
}

fn render_overview() -> VNode {
    rsx! {
        div { id: "page-system", class: "tairitsu-page",
            section { class: "page-section",
                h2 { class: "page-section__title", "System" }
                p { "系统架构文档，了解 Tairitsu 的核心设计和实现原理。" }
                div { class: "card-grid",
                    div { class: "card",
                        h3 { class: "card__title", "系统总览" }
                        p { class: "card__body", "整体架构分层和设计原则。" }
                        a {
                            href: "/system/overview",
                            class: "ts-btn ts-btn--secondary",
                            "阅读文档"
                        }
                    }
                    div { class: "card",
                        h3 { class: "card__title", "运行时" }
                        p { class: "card__body", "镜像/容器模型和执行引擎。" }
                        a {
                            href: "/system/runtime",
                            class: "ts-btn ts-btn--secondary",
                            "阅读文档"
                        }
                    }
                    div { class: "card",
                        h3 { class: "card__title", "WIT 流水线" }
                        p { class: "card__body", "W3C WebIDL 到 WIT 的生成流程。" }
                        a {
                            href: "/system/wit-pipeline",
                            class: "ts-btn ts-btn--secondary",
                            "阅读文档"
                        }
                    }
                    div { class: "card",
                        h3 { class: "card__title", "Web 后端" }
                        p { class: "card__body", "web 与 wit-bindings 双后端架构。" }
                        a {
                            href: "/system/web-backends",
                            class: "ts-btn ts-btn--secondary",
                            "阅读文档"
                        }
                    }
                }
            }
        }
    }
}
