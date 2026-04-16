use crate::components::breadcrumb;
use tairitsu_macros::rsx;
use tairitsu_vdom::{VElement, VNode};

fn el(tag: &str) -> VElement {
    VElement::new(tag)
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
                p { class: "card__body",
                    "Search input with optional icon, clear button, and debounced search callback support."
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Basic Search" }
                    div { class: "demo-block__body",
                        div { class: "search-wrapper", style: "max-width:400px;",
                            span { class: "search-icon", "\u{1F50D}" }
                            ..vec![search_basic]
                        }
                    }
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Search Sizes" }
                    div { class: "demo-block__body",
                        div { style: "display:flex;flex-direction:column;gap:var(--ts-spacing-md);max-width:400px;",
                            div { class: "search-wrapper",
                                span { class: "search-icon", style: "font-size:0.75rem;", "\u{1F50D}" }
                                ..vec![VNode::Element(el("input").attr("type", "text").attr("placeholder", "Small search").attr("style", "padding:4px 8px 4px 2rem;font-size:0.8125rem;"))]
                            }
                            div { class: "search-wrapper",
                                span { class: "search-icon", "\u{1F50D}" }
                                ..vec![search_docs]
                            }
                            div { class: "search-wrapper",
                                span { class: "search-icon", style: "font-size:1rem;", "\u{1F50D}" }
                                ..vec![VNode::Element(el("input").attr("type", "text").attr("placeholder", "Large search").attr("style", "padding:10px 12px 10px 2.75rem;font-size:1rem;"))]
                            }
                        }
                    }
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Search in Navigation Bar" }
                    div { class: "demo-block__body",
                        div { class: "search-wrapper", style: "max-width:100%;",
                            span { class: "search-icon", "\u{1F50D}" }
                            ..vec![search_api]
                        }
                        p { style: "font-size:0.8125rem;color:var(--hi-color-text-disabled);margin-top:8px;",
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
