//! Build, Test and Release Guide

use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

pub fn render() -> VNode {
    rsx! {
        div { id: "page-guides-build-test-release", class: "hikari-page",
            section { class: "page-section",
                h2 { class: "page-section__title",
                    "构建、测试与发布"
                }
                div { class: "hi-markdown-content",
                    p {
                        "了解 Tairitsu 的构建流程、测试策略和发布流程。"
                    }

                    h3 { "构建" }
                    p {
                        "使用 just 命令进行构建："
                    }
                    div { class: "hi-code-block language-bash",
                        pre {
                            code {
                                "# 开发构建
just dev

# 生产构建
just build-web

# 运行开发服务器
just serve-web"
                            }
                        }
                    }

                    h3 { "测试" }
                    p {
                        "运行测试来验证代码正确性："
                    }
                    div { class: "hi-code-block language-bash",
                        pre {
                            code {
                                "# 运行测试
just test

# 运行特定包测试
cargo test -p tairitsu-vdom"
                            }
                        }
                    }

                    h3 { "发布" }
                    p {
                        "发布前的检查和发布流程："
                    }
                    div { class: "hi-code-block language-bash",
                        pre {
                            code {
                                "# 发布检查
cargo publish --dry-run

# 发布到 crates.io
cargo publish"
                            }
                        }
                    }

                    h3 { "版本策略" }
                    p {
                        "Tairitsu 遵循语义化版本控制（SemVer）。在发布新版本时："
                    }
                    ul {
                        li { "主版本号：不兼容的 API 变更" }
                        li { "次版本号：向后兼容的功能新增" }
                        li { "修订号：向后兼容的问题修复" }
                    }

                    h3 { "变更日志" }
                    p {
                        "每个版本都应该包含变更日志，记录新增功能、问题修复和破坏性变更。"
                    }
                }
            }
        }
    }
}
