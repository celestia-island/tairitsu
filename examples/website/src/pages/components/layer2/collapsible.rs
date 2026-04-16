use crate::components::breadcrumb;
use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

pub fn render() -> VNode {
    rsx! {
        div { id: "page-component-collapsible", class: "hikari-page",
            ..vec![breadcrumb(&[("Home", "/"), ("Components", "/components"), ("Layer 2 \u{2014} Composed", "/components/layer2/collapsible"), ("Collapsible", "")])]
            section { class: "page-section",
                h2 { class: "page-section__title", "Collapsible" }
                p { class: "card__body",
                    "Expandable/collapsible content panel with smooth animation and custom trigger."
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Basic Collapsible" }
                    div { class: "demo-block__body",
                        details { class: "hi-collapsible",
                            summary { "Click to expand" }
                            p { "This content is hidden by default and shown when the user clicks the summary." }
                        }
                    }
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "FAQ Accordion" }
                    div { class: "demo-block__body",
                        details { class: "hi-collapsible",
                            summary { "What is Tairitsu?" }
                            p { "Tairitsu is a WASM Component Runtime Engine that provides a complete framework for building web applications using Rust compiled to WebAssembly." }
                        }
                        details { class: "hi-collapsible",
                            summary { "What runtime does it target?" }
                            p { "Tairitsu targets the wasm32-wasip2 target, using the WebAssembly Component Model for browser execution." }
                        }
                        details { class: "hi-collapsible",
                            summary { "How does routing work?" }
                            p { "The website uses a JavaScript-based SPA router with show/hide pages. All pages are rendered into the DOM at once, and only the active page is displayed." }
                        }
                        details { class: "hi-collapsible",
                            summary { "Can I use it with React/Vue?" }
                            p { "Tairitsu uses its own virtual DOM system (tairitsu-vdom) with React-like hooks. It is not designed to interoperate with React or Vue directly." }
                        }
                    }
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Nested Collapsible" }
                    div { class: "demo-block__body",
                        details { class: "hi-collapsible",
                            summary { "Project Structure" }
                            details { class: "hi-collapsible", style: "margin:8px 0;border:none;",
                                summary { "packages/" }
                                p { "Core framework packages: vdom, hooks, macros, web." }
                            }
                            details { class: "hi-collapsible", style: "margin:8px 0;border:none;",
                                summary { "examples/" }
                                p { "Example applications including the documentation website." }
                            }
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
                                tr { td { code { "open" } } td { code { "bool" } } td { "Controlled open state" } }
                                tr { td { code { "accordion" } } td { code { "bool" } } td { "Close others when opening" } }
                                tr { td { code { "trigger" } } td { code { "VNode" } } td { "Custom trigger element" } }
                                tr { td { code { "onChange" } } td { code { "(open: bool) => void" } } td { "Toggle callback" } }
                            }
                        }
                    }
                }
            }
        }
    }
}
