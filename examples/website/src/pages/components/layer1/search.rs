use crate::components::{breadcrumb, svg_icon};
use hikari_icons::MdiIcon;
use tairitsu_macros::rsx;
use tairitsu_vdom::{VElement, VNode, VText};

fn el(tag: &str) -> VElement {
    VElement::new(tag)
}

fn txt(s: &str) -> VNode {
    VNode::Text(VText::new(s))
}

fn search_icon(size: u32) -> VNode {
    svg_icon(MdiIcon::Magnify, size, "search-icon")
}

pub fn render() -> VNode {
    let search_basic = VNode::Element(
        el("input")
            .attr("type", "text")
            .attr("placeholder", "Search..."),
    );
    let search_docs = VNode::Element(
        el("input")
            .attr("type", "text")
            .attr("placeholder", "Search documentation..."),
    );
    let search_api = VNode::Element(
        el("input")
            .attr("type", "text")
            .attr("placeholder", "Search API reference..."),
    );

    rsx! {
        div { id: "page-component-search", class: "hikari-page",
            ..vec![breadcrumb(&[("Home", "/"), ("Components", "/components"), ("Layer 1 \u{2014} Base", "/components/layer1/search"), ("Search", "")])]
            section { class: "page-section",
                h2 { class: "page-section__title", "Search" }
                p { class: "page-section__description",
                    "Search input with optional icon, clear button, and debounced search callback support."
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Basic Search" }
                    div { class: "demo-block__body",
                        div { class: "search-wrapper", style: "max-width:400px;",
                            ..vec![search_icon(14), search_basic]
                        }
                    }
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Search Sizes" }
                    div { class: "demo-block__body",
                        div { style: "display:flex;flex-direction:column;gap:var(--ts-spacing-md);max-width:400px;",
                            div { class: "search-wrapper search-wrapper--sm",
                                ..vec![search_icon(12), VNode::Element(el("input").attr("type", "text").attr("placeholder", "Small search").class("hi-search-input-sm"))]
                            }
                            div { class: "search-wrapper",
                                ..vec![search_icon(14), search_docs]
                            }
                            div { class: "search-wrapper search-wrapper--lg",
                                ..vec![search_icon(18), VNode::Element(el("input").attr("type", "text").attr("placeholder", "Large search").class("hi-search-input-lg"))]
                            }
                        }
                    }
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Search in Navigation Bar" }
                    div { class: "demo-block__body",
                        div { class: "search-wrapper", style: "max-width:100%;",
                            ..vec![search_icon(14), search_api]
                        }
                        p { class: "demo-hint",
                            "Hint: Try searching for \"component\" or \"guide\""
                        }
                    }
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "API" }
                    div { class: "demo-block__body",
                        table { class: "api-table",
                            thead {
                                tr { th { "Property" } th { "Type" } th { "Description" } }
                            }
                            tbody {
                                tr { td { code { "placeholder" } } td { code { "string" } } td { "Placeholder text" } }
                                tr { td { code { "onSearch" } } td { code { "(query: string) => void" } } td { "Search callback" } }
                                tr { td { code { "debounce" } } td { code { "number" } } td { "Debounce delay in ms" } }
                                tr { td { code { "clearable" } } td { code { "bool" } } td { "Show clear button" } }
                                tr { td { code { "size" } } td { code { "small | default | large" } } td { "Input size" } }
                            }
                        }
                    }
                }
            }
        }
    }
}
