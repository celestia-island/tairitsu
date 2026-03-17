//! 404 Not Found page

use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

pub fn render() -> VNode {
    rsx! {
        div { id: "page-not-found", class: "tairitsu-page",
            div { class: "not-found",
                h1 { "404" }
                p { "页面未找到" }
                p {
                    "请求的页面不存在。请检查 URL 或返回首页。"
                }
                a { href: "#/", class: "ts-btn ts-btn--primary",
                    "返回首页"
                }
            }
        }
    }
}
