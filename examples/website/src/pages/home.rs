//! Home page — hero section and navigation cards.

use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

pub fn render() -> VNode {
    rsx! {
        div { id: "page-home", class: "tairitsu-page is-active",
            section { class: "page-hero",
                div { class: "page-hero__inner",
                    h1 { class: "page-hero__title", "Tairitsu" }
                    p { class: "page-hero__subtitle", "泛型 WASM Component Runtime 引擎" }
                    p { class: "page-hero__desc",
                        "采用 Docker-like 架构的通用运行时。支持镜像/容器模型管理 WASM 模块，提供灵活的构建器模式和 WIT-based 类型安全通信。"
                    }
                    div { class: "page-hero__actions",
                        a {
                            href: "/guides/quick-start",
                            class: "ts-btn ts-btn--primary ts-btn--lg",
                            "快速开始"
                        }
                        a {
                            href: "/system/overview",
                            class: "ts-btn ts-btn--secondary ts-btn--lg",
                            "系统架构"
                        }
                    }
                }
            }
            section { class: "page-section",
                h2 { class: "page-section__title", "什么是 Tairitsu?" }
                div { class: "card-grid",
                    div { class: "card",
                        h3 { class: "card__title", "镜像/容器模型" }
                        p { class: "card__body",
                            "使用 Docker-like 架构管理 WASM 模块。Image 是不可变的 WASM 组件二进制文件，Container 是运行实例。"
                        }
                    }
                    div { class: "card",
                        h3 { class: "card__title", "通用运行时" }
                        p { class: "card__body",
                            "不预设任何特定 WIT 接口，提供可插拔的宿主导入与客体导出调用，同时支持编译期与运行期接口路径。"
                        }
                    }
                    div { class: "card",
                        h3 { class: "card__title", "构建器模式" }
                        p { class: "card__body",
                            "灵活的 Container::builder() API，支持配置 host imports、guest initializer、linker state 和 execution context。"
                        }
                    }
                }
            }
            section { class: "page-section",
                h2 { class: "page-section__title", "架构分层" }
                div { class: "card-grid",
                    div { class: "card",
                        h3 { class: "card__title", "App Layer" }
                        p { class: "card__body",
                            "自定义 WIT 接口、业务组件、示例应用。"
                        }
                    }
                    div { class: "card",
                        h3 { class: "card__title", "Framework Layer" }
                        p { class: "card__body",
                            "runtime + macros + vdom/hooks/style/web + packager。"
                        }
                    }
                    div { class: "card",
                        h3 { class: "card__title", "Host Layer" }
                        p { class: "card__body",
                            "wasmtime/native host 和 browser-glue runtime adaptors。"
                        }
                    }
                }
            }
            section { class: "page-section",
                h2 { class: "page-section__title", "核心特性" }
                div { class: "card-grid",
                    div { class: "card",
                        h3 { class: "card__title", "接口先行" }
                        p { class: "card__body",
                            "优先通过 WIT 描述协议，确保类型安全和跨语言互操作性。"
                        }
                    }
                    div { class: "card",
                        h3 { class: "card__title", "运行时解耦" }
                        p { class: "card__body",
                            "容器模型不绑定业务语义，支持多种执行环境和宿主平台。"
                        }
                    }
                    div { class: "card",
                        h3 { class: "card__title", "双路径共存" }
                        p { class: "card__body",
                            "web 与 wit-bindings 两种后端可并行演进，满足不同场景需求。"
                        }
                    }
                    div { class: "card",
                        h3 { class: "card__title", "离线优先" }
                        p { class: "card__body",
                            "WIT 缓存支持无网构建，提升开发体验。"
                        }
                    }
                }
            }
        }
    }
}
