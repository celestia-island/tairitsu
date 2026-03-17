//! System Overview

use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

pub fn render() -> VNode {
    rsx! {
        div { id: "page-system-overview", class: "tairitsu-page",
            section { class: "page-section",
                h2 { class: "page-section__title",
                    "系统架构总览"
                }
                div { class: "doc-content",
                    p {
                        "Tairitsu 是面向 WebAssembly Component Model 的通用运行时，核心目标是："
                    }
                    ul {
                        li { "不绑定单一业务 WIT" }
                        li { "提供可插拔宿主导入与客体导出调用" }
                        li { "同时支持编译期与运行期接口路径" }
                    }

                    h3 { "架构分层" }
                    div { class: "code-block language-text",
                        pre {
                            code {
                                "应用层（业务）
  ├─ 自定义 WIT 接口 / 组件
  └─ 示例工程（examples/*）

框架层（Tairitsu）
  ├─ runtime: 镜像/容器/调用引擎
  ├─ macros: 宏辅助接口定义
  ├─ vdom + hooks + web: UI 运行层
  └─ packager: 解析与分发

宿主层
  ├─ wasmtime / native host
  └─ browser-glue（TS）"
                            }
                        }
                    }

                    h3 { "关键设计原则" }
                    ol {
                        li {
                            strong { "接口先行：" }
                            "优先通过 WIT 描述协议"
                        }
                        li {
                            strong { "运行时解耦：" }
                            "容器模型不绑定业务语义"
                        }
                        li {
                            strong { "双路径共存：" }
                            "web 与 wit-bindings 可并行演进"
                        }
                        li {
                            strong { "离线优先：" }
                            "WIT 缓存支持无网构建"
                        }
                    }

                    h3 { "推荐阅读路径" }
                    ul {
                        li { "新用户：快速开始 → 系统总览 → 运行时与容器模型" }
                        li { "浏览器方向：迁移说明 → Web 平台双后端 → WIT 流水线" }
                        li { "维护者：工作区地图 → 包清单 → 版本策略" }
                    }
                }
            }
        }
    }
}
