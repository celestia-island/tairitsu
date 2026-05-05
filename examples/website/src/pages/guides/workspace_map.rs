//! Workspace Map Guide

use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

use crate::components::breadcrumb;

pub fn render() -> VNode {
    rsx! {
        div { id: "page-guides-workspace-map", class: "ts-page",
            ..vec![breadcrumb(&[("Home", "/"), ("Guides", "/guides"), ("Workspace Map", "")])]
            section { class: "page-section",
                h2 { class: "page-section__title", "工作区地图" }
                div { class: "hi-markdown-content",
                    p { "了解 Tairitsu 项目的目录结构和各个包的职责。" }

                    h3 { "目录结构" }
                    div { class: "hi-code-block",
                        pre { class: "hi-code-content",
                            "graph TD\n    PKG[\"packages/ — Rust 核心包与工具包\"] --> RT[\"runtime/ — Image, Container, builder API\"]\n    PKG --> MC[\"macros/ — rsx!, wit_world! 宏\"]\n    PKG --> VD[\"vdom/ — 平台无关 VDOM\"]\n    PKG --> HK[\"hooks/ — React Hooks 风格的 hooks\"]\n    PKG --> WB[\"web/ — Web 平台绑定\"]\n    PKG --> BG[\"browser-glue/ — 浏览器适配器\"]\n    PKG --> PK[\"packager/ — 组件打包器\"]\n    EX[\"examples/ — 示例工程（不同 WIT 集成路径）\"]\n    SC[\"scripts/ — WebIDL/WIT 生成与辅助脚本\"]\n    DC[\"docs/ — 项目文档（多语言）\"]\n    TS[\"tests/ — 端到端相关资产\"]"
                        }
                    }

                    h3 { "核心包说明" }

                    h4 { "packages/runtime" }
                    p {
                        "核心运行时引擎。包含 Image（不可变 WASM 组件二进制）、Container（运行实例）、builder API 和 Wasmtime component 执行引擎。"
                    }

                    h4 { "packages/macros" }
                    p {
                        "过程宏。提供 rsx! 宏用于声明式 UI 语法，wit_world! 宏用于 WIT 接口定义。"
                    }

                    h4 { "packages/vdom" }
                    p {
                        "平台无关的虚拟 DOM。定义 VNode、VElement、VText 等核心类型，以及样式、类和事件模型。"
                    }

                    h4 { "packages/hooks" }
                    p {
                        "React Hooks 风格的状态管理。提供 use_state、use_signal、use_effect 等 hooks。"
                    }

                    h4 { "packages/web" }
                    p {
                        "Web 平台绑定。提供 WebPlatform 和 WitPlatform 两种浏览器后端实现。"
                    }

                    h4 { "packages/packager" }
                    p {
                        "组件打包器。CLI 构建编排、开发服务器、资源复制和组件宿主 HTML 生成。"
                    }
                }
            }
        }
    }
}
