//! Guides pages - documentation for getting started and development

pub mod build_test_release;
pub mod glossary;
pub mod migration;
pub mod quick_start;
pub mod workspace_map;

use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

/// Render all guide pages
pub fn render_all() -> Vec<VNode> {
    vec![
        render_overview(),
        quick_start::render(),
        workspace_map::render(),
        build_test_release::render(),
        migration::render(),
        glossary::render(),
    ]
}

fn render_overview() -> VNode {
    rsx! {
        div { id: "page-guides", class: "hikari-page",
            section { class: "page-section",
                h2 { class: "page-section__title", "Guides" }
                p { class: "card__body",
                    "Tairitsu 的使用指南覆盖从入门到架构、从开发到发布的完整链路。"
                }
                div { class: "card-grid",
                    div { class: "card",
                        h3 { class: "card__title", "快速开始" }
                        p { class: "card__body",
                            "安装工具、构建、测试、运行示例的完整指南。"
                        }
                        a {
                            href: "/guides/quick-start",
                            class: "hi-button hi-button-secondary",
                            "阅读文档"
                        }
                    }
                    div { class: "card",
                        h3 { class: "card__title", "工作区地图" }
                        p { class: "card__body", "了解项目结构和各个包的职责。" }
                        a {
                            href: "/guides/workspace-map",
                            class: "hi-button hi-button-secondary",
                            "阅读文档"
                        }
                    }
                    div { class: "card",
                        h3 { class: "card__title", "构建、测试与发布" }
                        p { class: "card__body", "完整的构建流程和发布策略。" }
                        a {
                            href: "/guides/build-test-release",
                            class: "hi-button hi-button-secondary",
                            "阅读文档"
                        }
                    }
                    div { class: "card",
                        h3 { class: "card__title", "迁移指南" }
                        p { class: "card__body", "从其他框架迁移到 Tairitsu 的指南。" }
                        a {
                            href: "/guides/migration",
                            class: "hi-button hi-button-secondary",
                            "阅读文档"
                        }
                    }
                }
            }
        }
    }
}
