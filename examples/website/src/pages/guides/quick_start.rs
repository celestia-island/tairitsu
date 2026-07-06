//! Quick Start Guide

use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

use crate::components::breadcrumb;

pub fn render() -> VNode {
    rsx! {
        div { id: "page-guides-quick-start", class: "hikari-page",
            ..vec![breadcrumb(&[("Home", "/"), ("Guides", "/guides"), ("Quick Start", "")])]
            section { class: "page-section",
                h2 { class: "page-section__title",
                    "快速开始"
                }
                div { class: "hi-markdown-content",
                    p {
                        "本指南帮助你在本地完成最小可用链路：安装工具、构建、测试、运行示例。"
                    }

                    h3 { "1. 环境准备" }
                    p {
                        "需要准备以下环境："
                    }
                    ul {
                        li { "Rust（建议 stable）" }
                        li { "`just` 命令工具" }
                        li { "Python 3（用于 WIT 生成脚本）" }
                        li { "Node.js（用于 packages/browser-glue）" }
                    }
                    div { class: "hi-code-block language-bash",
                        pre {
                            code {
                                "# 安装 just 命令工具
cargo install just

# 安装工具
just install-tools"
                            }
                        }
                    }

                    h3 { "2. 一次性全量校验" }
                    p {
                        "该命令会运行核心编译与检查流程，用于快速确认环境与依赖可用。"
                    }
                    div { class: "hi-code-block language-bash",
                        pre {
                            code {
                                "# 一次性全量校验
    just test"
                            }
                        }
                    }

                    h3 { "3. 运行示例" }
                    p {
                        "运行各种示例来了解框架的不同特性："
                    }
                    div { class: "hi-code-block language-bash",
                        pre {
                            code {
                                "# 宏驱动示例
just run-macro-demo

# trait 驱动示例
just run-simple-demo

# 动态调用示例
just run-dynamic-advanced"
                            }
                        }
                    }

                    h3 { "4. 浏览器 WIT 路线（可选）" }
                    p {
                        "若你计划使用 Component Model 浏览器接口："
                    }
                    div { class: "hi-code-block language-bash",
                        pre {
                            code {
                                "# 浏览器 WIT 路线
    cargo check -p tairitsu-web --features wit-bindings"
                            }
                        }
                    }

                    h3 { "5. 常见问题" }

                    h4 { "找不到 wasm32-wasip2 target" }
                    p {
                        "执行以下命令添加 target："
                    }
                    div { class: "hi-code-block language-bash",
                        pre {
                            code {
                                "# 添加 wasm32-wasip2 target
    rustup target add wasm32-wasip2"
                            }
                        }
                    }

                    h4 { "Python 脚本报错" }
                    p {
                        "优先确认："
                    }
                    ul {
                        li { "python3 可执行" }
                        li { "网络可访问 w3c/webref" }
                        li { "项目目录可写" }
                    }

                    h4 { "TypeScript 检查失败" }
                    p {
                        "在 packages/browser-glue 中执行："
                    }
                    div { class: "hi-code-block language-bash",
                        pre {
                            code {
                                "npm install
    npm run typecheck"
                            }
                        }
                    }
                }
            }
        }
    }
}
