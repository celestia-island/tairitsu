//! Migration Guide

use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

use crate::components::breadcrumb;

pub fn render() -> VNode {
    rsx! {
        div { id: "page-guides-migration", class: "hikari-page",
            ..vec![breadcrumb(&[("Home", "/"), ("Guides", "/guides"), ("Migration", "")])]
            section { class: "page-section",
                h2 { class: "page-section__title",
                    "迁移指南"
                }
                div { class: "hi-markdown-content",
                    p {
                        "帮助你从其他框架迁移到 Tairitsu。"
                    }

                    h3 { "从 Dioxus 迁移" }
                    p {
                        "Tairitsu 的 rsx! 宏设计参考了 Dioxus，大部分语法可以直接复用："
                    }
                    div { class: "hi-code-block language-rust",
                        pre {
                            code {
                                "// Dioxus 0.7 风格
rsx! {
    div {
        class: \"container\",
        onclick: move |_| {},
        \"Hello\"
    }
}

// Tairitsu 风格（几乎相同）
rsx! {
    div {
        class: \"container\",
        onclick: move |_| {},
        \"Hello\"
    }
}"
                            }
                        }
                    }

                    h3 { "从 web-sys 迁移到 WIT 绑定" }
                    p {
                        "从命令式 DOM 操作迁移到声明式 rsx! 宏："
                    }
                    div { class: "hi-code-block language-rust",
                        pre {
                            code {
                                "// web-sys 风格
let button = document.create_element(\"button\")?;
button.set_class_name(\"btn\");
button.set_text_content(Some(\"Click\"));

// Tairitsu rsx! 风格
rsx! {
    button {
        class: \"btn\",
        onclick: |e| handle_click(e),
        \"Click\"
    }
}"
                            }
                        }
                    }

                    h3 { "状态管理迁移" }
                    p {
                        "Tairitsu 提供 React Hooks 风格的状态管理："
                    }
                    ul {
                        li { "use_state - 简单状态管理" }
                        li { "use_signal - 响应式信号" }
                        li { "use_effect - 副作用处理" }
                    }

                    h3 { "注意事项" }
                    ul {
                        li { "确保 Rust 版本 >= 1.75" }
                        li { "添加 wasm32-wasip2 target" }
                        li { "检查第三方依赖的兼容性" }
                    }
                }
            }
        }
    }
}
