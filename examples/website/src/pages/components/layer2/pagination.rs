use crate::components::breadcrumb;
use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

pub fn render() -> VNode {
    rsx! {
        div { id: "page-component-pagination", class: "hikari-page",
            ..vec![breadcrumb(&[("Home", "/"), ("Components", "/components"), ("Layer 2 — Composed", "/components/layer2/pagination"), ("Pagination", "")])]
            section { class: "page-section",
                h2 { class: "page-section__title", "Pagination" }
                p { class: "card__body",
                    "Page navigation component with page size selector, jumper, total count, and mini/standard variants."
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Basic Pagination" }
                    div { class: "demo-block__body",
                        nav { class: "hi-pagination",
                            a { href: "#", class: "hi-pagination-prev", "\u{2039}" }
                            a { href: "#", class: "hi-pagination-item hi-pagination-active", "1" }
                            a { href: "#", class: "hi-pagination-item", "2" }
                            a { href: "#", class: "hi-pagination-item", "3" }
                            span { class: "hi-pagination-ellipsis", "..." }
                            a { href: "#", class: "hi-pagination-next", "\u{203A}" }
                        }
                    }
                }
            }
        }
    }
}
