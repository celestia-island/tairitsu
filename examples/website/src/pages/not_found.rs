//! 404 Not Found page

use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

pub fn render() -> VNode {
    rsx! {
        div { id: "page-not-found", class: "hikari-page",
            div { class: "hi-container hi-container-md",
                section { class: "hi-section hi-section-lg",
                    div { class: "hi-section-body",
                        div { class: "hi-text-center",
                            h1 { class: "hi-text-2xl hi-text-secondary hi-mb-6", "404" }
                            p { class: "hi-text-lg hi-text-primary", "页面未找到" }
                            p { class: "hi-text-sm hi-text-primary",
                                "请求的页面不存在。请检查 URL 或返回首页。"
                            }
                            div { style: "height:2rem" }
                            a {
                                href: "/",
                                class: "hi-button hi-button-primary hi-button-md hi-button-width-auto hi-justify-center",
                                "返回首页"
                            }
                        }
                    }
                }
            }
        }
    }
}
