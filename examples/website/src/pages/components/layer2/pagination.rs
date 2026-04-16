use crate::components::breadcrumb;
use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

pub fn render() -> VNode {
    rsx! {
        div { id: "page-component-pagination", class: "hikari-page",
            ..vec![breadcrumb(&[("Home", "/"), ("Components", "/components"), ("Layer 2 \u{2014} Composed", "/components/layer2/pagination"), ("Pagination", "")])]
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
                            a { href: "#", class: "hi-pagination-item", "10" }
                            a { href: "#", class: "hi-pagination-next", "\u{203A}" }
                        }
                    }
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Pagination with Info" }
                    div { class: "demo-block__body",
                        div { style: "display:flex;align-items:center;justify-content:space-between;flex-wrap:wrap;gap:12px;",
                            nav { class: "hi-pagination",
                                a { href: "#", class: "hi-pagination-prev", "\u{2039}" }
                                a { href: "#", class: "hi-pagination-item hi-pagination-active", "1" }
                                a { href: "#", class: "hi-pagination-item", "2" }
                                a { href: "#", class: "hi-pagination-item", "3" }
                                a { href: "#", class: "hi-pagination-item", "4" }
                                a { href: "#", class: "hi-pagination-item", "5" }
                                a { href: "#", class: "hi-pagination-next", "\u{203A}" }
                            }
                            span { style: "font-size:0.8125rem;color:var(--hi-color-text-disabled);",
                                "Showing 1-10 of 50 items"
                            }
                        }
                    }
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Mini Pagination" }
                    div { class: "demo-block__body",
                        div { style: "display:flex;flex-direction:column;gap:12px;",
                            nav { class: "hi-pagination",
                                a { href: "#", class: "hi-pagination-prev", style: "min-width:24px;height:24px;font-size:0.75rem;", "\u{2039}" }
                                a { href: "#", class: "hi-pagination-item hi-pagination-active", style: "min-width:24px;height:24px;font-size:0.75rem;", "1" }
                                a { href: "#", class: "hi-pagination-item", style: "min-width:24px;height:24px;font-size:0.75rem;", "2" }
                                a { href: "#", class: "hi-pagination-item", style: "min-width:24px;height:24px;font-size:0.75rem;", "3" }
                                a { href: "#", class: "hi-pagination-next", style: "min-width:24px;height:24px;font-size:0.75rem;", "\u{203A}" }
                            }
                        }
                    }
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "API" }
                    div { class: "demo-block__body",
                        table { class: "api-table",
                            thead {
                                tr { th { "Property" } th { "Type" } th { "Default" } th { "Description" } }
                            }
                            tbody {
                                tr { td { code { "current" } } td { code { "number" } } td { code { "1" } } td { "Current page" } }
                                tr { td { code { "total" } } td { code { "number" } } td { code { "0" } } td { "Total pages" } }
                                tr { td { code { "pageSize" } } td { code { "number" } } td { code { "10" } } td { "Items per page" } }
                                tr { td { code { "showSizeChanger" } } td { code { "bool" } } td { code { "false" } } td { "Show page size selector" } }
                                tr { td { code { "showQuickJumper" } } td { code { "bool" } } td { code { "false" } } td { "Show quick jump input" } }
                                tr { td { code { "onChange" } } td { code { "(page: number) => void" } } td { "-" } td { "Page change callback" } }
                            }
                        }
                    }
                }
            }
        }
    }
}
