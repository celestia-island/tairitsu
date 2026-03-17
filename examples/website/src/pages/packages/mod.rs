//! Packages pages - package documentation

use tairitsu_macros::rsx;
use tairitsu_vdom::{VElement, VNode, VText};

fn txt(s: &str) -> VNode {
    VNode::Text(VText::new(s))
}

/// Render all packages pages
pub fn render_all() -> Vec<VNode> {
    vec![render_overview(), render_list()]
}

fn render_overview() -> VNode {
    rsx! {
        div { id: "page-packages-overview", class: "tairitsu-page",
            section { class: "page-section",
                h2 { class: "page-section__title",
                    "Packages"
                }
                div { class: "doc-content",
                    p {
                        "Tairitsu 由多个核心包组成，每个包负责特定的功能领域。"
                    }

                    h3 { "包分层" }

                    h4 { "Layer 1 - Core" }
                    ul {
                        li {
                            strong { "runtime：" }
                            "镜像/容器模型，Wasmtime 执行引擎"
                        }
                        li {
                            strong { "vdom：" }
                            "平台无关的虚拟 DOM"
                        }
                    }

                    h4 { "Layer 2 - Platform" }
                    ul {
                        li {
                            strong { "web：" }
                            "Web 平台绑定（web-sys + wit-bindings）"
                        }
                        li {
                            strong { "hooks：" }
                            "React Hooks 风格的状态管理"
                        }
                    }

                    h4 { "Layer 3 - Tools" }
                    ul {
                        li {
                            strong { "macros：" }
                            "rsx! 宏，wit_world! 宏"
                        }
                        li {
                            strong { "packager：" }
                            "CLI 构建工具，开发服务器"
                        }
                    }
                }
            }
        }
    }
}

fn render_list() -> VNode {
    rsx! {
        div { id: "page-packages-list", class: "tairitsu-page",
            section { class: "page-section",
                h2 { class: "page-section__title",
                    "包清单"
                }
                div { class: "doc-content",
                    h3 { "packages/runtime" }
                    p {
                        "核心运行时引擎。提供 Image（镜像）、Container（容器）、Builder API 和 Wasmtime component 执行能力。"
                    }

                    h3 { "packages/vdom" }
                    p {
                        "平台无关的虚拟 DOM。定义 VNode、VElement、VText、Classes、Style 等核心类型。"
                    }

                    h3 { "packages/hooks" }
                    p {
                        "React Hooks 风格的状态管理。提供 use_state、use_signal、use_effect、use_memo 等 hooks。"
                    }

                    h3 { "packages/macros" }
                    p {
                        "过程宏。rsx! 宏用于声明式 UI，wit_world! 宏用于 WIT 接口定义。"
                    }

                    h3 { "packages/web" }
                    p {
                        "Web 平台绑定。WebPlatform（web-sys）和 WitPlatform（wit-bindings）两种后端实现。"
                    }

                    h3 { "packages/browser-glue" }
                    p {
                        "浏览器适配器。TypeScript 实现的宿主导入和客体导出绑定。"
                    }

                    h3 { "packages/packager" }
                    p {
                        "组件打包器。CLI 构建编排、开发服务器、资源复制和组件宿主 HTML 生成。"
                    }
                }
            }
        }
    }
}
