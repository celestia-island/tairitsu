//! Versioning Documentation

use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

use crate::components::breadcrumb;

pub fn render() -> VNode {
    rsx! {
        div { id: "page-system-versioning", class: "ts-page",
            ..vec![breadcrumb(&[("Home", "/"), ("System", "/system"), ("Versioning", "")])]
            section { class: "page-section",
                h2 { class: "page-section__title",
                    "版本与兼容性策略"
                }
                div { class: "hi-markdown-content",
                    p {
                        "Tairitsu 遵循语义化版本控制（SemVer），确保 API 的稳定性和可预测性。"
                    }

                    h3 { "版本号格式" }
                    p {
                        "版本号格式为 MAJOR.MINOR.PATCH，例如 0.1.0"
                    }
                    ul {
                        li {
                            strong { "MAJOR：" }
                            "不兼容的 API 变更"
                        }
                        li {
                            strong { "MINOR：" }
                            "向后兼容的功能新增"
                        }
                        li {
                            strong { "PATCH：" }
                            "向后兼容的问题修复"
                        }
                    }

                    h3 { "依赖声明" }
                    div { class: "hi-code-block language-toml",
                        pre {
                            code {
                                "# Cargo.toml
[dependencies]
tairitsu-vdom = \"0.1\"
tairitsu-hooks = \"0.1\"
tairitsu-macros = \"0.1\""
                            }
                        }
                    }

                    h3 { "发布周期" }
                    ul {
                        li { "PATCH 版本：按需发布，修复紧急问题" }
                        li { "MINOR 版本：每月或功能集成就绪时发布" }
                        li { "MAJOR 版本：重大架构变更时发布" }
                    }

                    h3 { "兼容性保证" }
                    p {
                        "在 0.x 版本期间，API 可能会有变化。我们会："
                    }
                    ul {
                        li { "在 CHANGELOG 中详细记录所有变更" }
                        li { "废弃 API 至少保留一个 MINOR 版本" }
                        li { "移除前会在编译时发出警告" }
                    }

                    h3 { "WIT 版本兼容" }
                    p {
                        "WIT 接口的变更遵循以下规则："
                    }
                    ul {
                        li { "新增接口：向后兼容" }
                        li { "修改接口签名：需要 MAJOR 版本升级" }
                        li { "移除接口：需要 MAJOR 版本升级" }
                    }
                }
            }
        }
    }
}
