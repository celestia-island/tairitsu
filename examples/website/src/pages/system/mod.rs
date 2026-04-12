//! System pages - architecture and runtime documentation

pub mod overview;
pub mod runtime;
pub mod versioning;
pub mod web_backends;
pub mod wit_pipeline;

use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

use crate::components::breadcrumb;

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
        div { id: "page-system", class: "hikari-page",
            ..vec![breadcrumb(&[("Home", "/"), ("System", "")])]
            section { class: "page-section",
                h2 { class: "page-section__title", "System" }
                p { "系统架构文档，了解 Tairitsu 的核心设计和实现原理。" }
                div { class: "card-grid",
                    div { class: "card",
                        h3 { class: "card__title", "System Overview" }
                        p { class: "card__body", "整体架构分层和设计原则。" }
                        a { href: "/system/overview", class: "hi-button hi-button-secondary", "阅读文档" }
                    }
                    div { class: "card",
                        h3 { class: "card__title", "Runtime Engine" }
                        p { class: "card__body", "镜像/容器模型和执行引擎。" }
                        a { href: "/system/runtime", class: "hi-button hi-button-secondary", "阅读文档" }
                    }
                    div { class: "card",
                        h3 { class: "card__title", "WIT Pipeline" }
                        p { class: "card__body", "W3C WebIDL 到 WIT 的生成流程。" }
                        a { href: "/system/wit-pipeline", class: "hi-button hi-button-secondary", "阅读文档" }
                    }
                    div { class: "card",
                        h3 { class: "card__title", "Web Backends" }
                        p { class: "card__body", "web 与 wit-bindings 双后端架构。" }
                        a { href: "/system/web-backends", class: "hi-button hi-button-secondary", "阅读文档" }
                    }
                    div { class: "card",
                        h3 { class: "card__title", "Versioning" }
                        p { class: "card__body", "语义化版本控制策略。" }
                        a { href: "/system/versioning", class: "hi-button hi-button-secondary", "阅读文档" }
                    }
                }
            }
        }
    }
}
