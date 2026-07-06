//! Web Backends Documentation

use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

use crate::components::breadcrumb;

pub fn render() -> VNode {
    rsx! {
        div { id: "page-system-web-backends", class: "hikari-page",
            ..vec![breadcrumb(&[("Home", "/"), ("System", "/system"), ("Web Backends", "")])]
            section { class: "page-section",
                h2 { class: "page-section__title",
                    "Web 平台双后端"
                }
                div { class: "hi-markdown-content",
                    p {
                        "Tairitsu 提供两种 Web 平台后端，满足不同的使用场景。"
                    }

                    h3 { "后端对比" }
                    table {
                        thead {
                            tr {
                                th { "特性" }
                                th { "web" }
                                th { "wit-bindings" }
                            }
                        }
                        tbody {
                            tr {
                                td { "目标平台" }
                                td { "wasm32-unknown-unknown" }
                                td { "wasm32-wasip2" }
                            }
                            tr {
                                td { "接口类型" }
                                td { "web-sys" }
                                td { "WIT Component Model" }
                            }
                            tr {
                                td { "调用方式" }
                                td { "JS glue" }
                                td { "Native host" }
                            }
                            tr {
                                td { "适用场景" }
                                td { "传统 Web 应用" }
                                td { "Component 生态" }
                            }
                        }
                    }

                    h3 { "web 后端" }
                    p {
                        "直接 DOM 操作，适合传统 Web 应用："
                    }
                    div { class: "hi-code-block language-rust",
                        pre {
                            code {
                                "// web 后端 - 直接 DOM 操作
let platform = WebPlatform::new()?;
let element = platform.create_element(\"div\")?;
element.set_text_content(\"Hello\");
platform.append_child(&element)?;"
                            }
                        }
                    }

                    h3 { "wit-bindings 后端" }
                    p {
                        "Component Model 原生，适合 Component 生态："
                    }
                    div { class: "hi-code-block language-rust",
                        pre {
                            code {
                                "// wit-bindings 后端 - Component Model
let platform = WitPlatform::new()?;
let element = rsx! {
    div { \"Hello\" }
};
platform.render(element)?;"
                            }
                        }
                    }

                    h3 { "选择建议" }
                    ul {
                        li {
                            strong { "选择 web：" }
                            "需要兼容现有 web-sys 代码库"
                        }
                        li {
                            strong { "选择 wit-bindings：" }
                            "需要与其他 WASM 组件互操作"
                        }
                        li {
                            strong { "两者共存：" }
                            "渐进式迁移策略"
                        }
                    }
                }
            }
        }
    }
}
