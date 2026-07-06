//! WIT Pipeline Documentation

use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

use crate::components::breadcrumb;

pub fn render() -> VNode {
    rsx! {
        div { id: "page-system-wit-pipeline", class: "hikari-page",
            ..vec![breadcrumb(&[("Home", "/"), ("System", "/system"), ("WIT Pipeline", "")])]
            section { class: "page-section",
                h2 { class: "page-section__title",
                    "WIT 流水线"
                }
                div { class: "hi-markdown-content",
                    p {
                        "从 W3C WebIDL 到 WIT 接口的自动化生成流程。"
                    }

                    h3 { "流水线步骤" }
                    ol {
                        li {
                            strong { "01 - Fetch specs：" }
                            "从 w3c/webref 拉取 IDL 规范"
                        }
                        li {
                            strong { "02 - Parse WebIDL：" }
                            "解析 IDL 语法树"
                        }
                        li {
                            strong { "03 - Generate WIT：" }
                            "转换为域 WIT 文件"
                        }
                        li {
                            strong { "04 - Compose world：" }
                            "组合 phase-0 与生成的域"
                        }
                    }

                    h3 { "WIT 接口示例" }
                    div { class: "hi-code-block language-wit",
                        pre {
                            code {
                                "// 生成的 WIT 接口示例
interface dom-element {
    get-attribute: func(name: string) -> option<string>;
    set-attribute: func(name: string, value: string) -> unit;
    remove-attribute: func(name: string) -> unit;
}

interface dom-document {
    get-element-by-id: func(id: string) -> option<dom-element>;
    create-element: func(tag: string) -> dom-element;
}"
                            }
                        }
                    }

                    h3 { "离线构建支持" }
                    p {
                        "WIT 文件会缓存到本地，支持无网络环境下的构建："
                    }
                    ul {
                        li { "首次构建时自动拉取并缓存" }
                        li { "后续构建优先使用缓存" }
                        li { "支持手动更新缓存" }
                    }

                    h3 { "自定义 WIT" }
                    p {
                        "项目可以定义自己的 WIT 接口："
                    }
                    ol {
                        li { "在 wit/ 目录下创建 .wit 文件" }
                        li { "在 Cargo.toml 中配置 wit-world" }
                        li { "使用 wit_world! 宏生成绑定" }
                    }
                }
            }
        }
    }
}
