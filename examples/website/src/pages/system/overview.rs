//! System Overview

use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

pub fn render() -> VNode {
    rsx! {
        div { id: "page-system-overview", class: "tairitsu-page",
            section { class: "page-section",
                h2 { class: "page-section__title", "系统架构总览" }
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
                    div { class: "mermaid",
                        "graph TD
    subgraph APP[\"应用层（业务）\"]
        A1[\"自定义 WIT 接口 / 组件\"]
        A2[\"示例工程（examples/*）\"]
    end
    subgraph FW[\"框架层（Tairitsu）\"]
        F1[\"runtime: 镜像/容器/调用引擎\"]
        F2[\"macros: 宏辅助接口定义\"]
        F3[\"vdom + hooks + web: UI 运行层\"]
        F4[\"packager: 解析与分发\"]
    end
    subgraph HOST[\"宿主层\"]
        H1[\"wasmtime / native host\"]
        H2[\"browser-glue（TS）\"]
    end
    APP --> FW --> HOST"
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
