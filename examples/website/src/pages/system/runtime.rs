//! Runtime Documentation

use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

use crate::components::breadcrumb;

pub fn render() -> VNode {
    rsx! {
        div { id: "page-system-runtime", class: "hikari-page",
            ..vec![breadcrumb(&[("Home", "/"), ("System", "/system"), ("Runtime", "")])]
            section { class: "page-section",
                h2 { class: "page-section__title",
                    "运行时与容器模型"
                }
                div { class: "hi-markdown-content",
                    p {
                        "Tairitsu 运行时采用 Docker-like 的镜像/容器架构，管理 WASM 组件的生命周期。"
                    }

                    h3 { "核心概念" }

                    h4 { "Image（镜像）" }
                    p {
                        "不可变的 WASM 组件二进制加上元数据。镜像包含了组件的代码、导入导出声明和配置信息。"
                    }

                    h4 { "Container（容器）" }
                    p {
                        "运行中的组件实例。封装了 store、linker 和调用上下文，提供安全的隔离环境。"
                    }

                    h4 { "Builder" }
                    p {
                        "容器构建器。用于配置宿主导入、客体初始化器、链接状态和执行上下文。"
                    }

                    h3 { "使用示例" }
                    div { class: "hi-code-block language-rust",
                        pre {
                            code {
                                "// 创建镜像
let image = Image::from_file(\"component.wasm\")?;

// 构建容器
let container = Container::builder(image)
    .with_host_imports(my_imports)
    .with_guest_initializer(init_fn)
    .build()?;

// 调用导出函数
let result: Result<Output, Error> = container.invoke(\"process\", &input)?;"
                            }
                        }
                    }

                    h3 { "执行流程" }
                    ol {
                        li {
                            strong { "加载镜像：" }
                            "从文件或字节加载 WASM 组件"
                        }
                        li {
                            strong { "构建容器：" }
                            "配置导入和初始化器"
                        }
                        li {
                            strong { "实例化：" }
                            "创建运行时实例"
                        }
                        li {
                            strong { "调用：" }
                            "执行导出函数"
                        }
                    }

                    h3 { "调用模式" }
                    p {
                        "支持两种调用模式："
                    }
                    ul {
                        li {
                            strong { "Typed Bindings：" }
                            "编译时类型安全，适用于固定契约"
                        }
                        li {
                            strong { "Dynamic ABI：" }
                            "运行时动态调用，支持插件式访问"
                        }
                    }
                }
            }
        }
    }
}
