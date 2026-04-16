use crate::components::breadcrumb;
use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

pub fn render() -> VNode {
    rsx! {
        div { id: "page-component-tree", class: "hikari-page",
            ..vec![breadcrumb(&[("Home", "/"), ("Components", "/components"), ("Layer 2 \u{2014} Composed", "/components/layer2/tree"), ("Tree", "")])]
            section { class: "page-section",
                h2 { class: "page-section__title", "Tree" }
                p { class: "card__body",
                    "Hierarchical tree view component with expand/collapse, checkbox selection, drag-and-drop, and async loading."
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Basic Tree" }
                    div { class: "demo-block__body",
                        ul { class: "hi-tree",
                            li { "\u{25B6} Root"
                                ul {
                                    li { "\u{25B6} Child 1"
                                        ul { li { "Leaf A" } li { "Leaf B" } }
                                    }
                                    li { "\u{25B6} Child 2"
                                        ul { li { "Leaf C" } }
                                    }
                                }
                            }
                        }
                    }
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "File Tree" }
                    div { class: "demo-block__body",
                        ul { class: "hi-tree",
                            li { "\u{25B6} packages"
                                ul {
                                    li { "\u{25B6} vdom"
                                        ul { li { "src/lib.rs" } li { "Cargo.toml" } }
                                    }
                                    li { "\u{25B6} hooks"
                                        ul { li { "src/lib.rs" } li { "Cargo.toml" } }
                                    }
                                    li { "\u{25B6} macros"
                                        ul { li { "src/lib.rs" } li { "Cargo.toml" } }
                                    }
                                }
                            }
                            li { "\u{25B6} examples"
                                ul {
                                    li { "\u{25B6} website"
                                        ul { li { "src/app.rs" } li { "Cargo.toml" } }
                                    }
                                }
                            }
                            li { "Cargo.toml" }
                            li { "justfile" }
                        }
                    }
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Tree with Icons" }
                    div { class: "demo-block__body",
                        ul { class: "hi-tree",
                            li { style: "display:flex;align-items:center;gap:6px;",
                                span { style: "color:var(--hi-color-accent);", "\u{1F4C1}" }
                                "\u{25B6} src"
                                ul {
                                    li { style: "display:flex;align-items:center;gap:6px;",
                                        span { style: "color:var(--hi-color-secondary);", "\u{1F4C4}" }
                                        "lib.rs"
                                    }
                                    li { style: "display:flex;align-items:center;gap:6px;",
                                        span { style: "color:var(--hi-color-secondary);", "\u{1F4C4}" }
                                        "app.rs"
                                    }
                                    li { style: "display:flex;align-items:center;gap:6px;",
                                        span { style: "color:var(--hi-color-accent);", "\u{1F4C1}" }
                                        "\u{25B6} pages"
                                        ul {
                                            li { style: "display:flex;align-items:center;gap:6px;",
                                                span { style: "color:var(--hi-color-secondary);", "\u{1F4C4}" }
                                                "home.rs"
                                            }
                                        }
                                    }
                                }
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
                                tr { td { code { "data" } } td { code { "TreeNode[]" } } td { "Tree data with children" } }
                                tr { td { code { "checkable" } } td { code { "bool" } } td { "Show checkboxes" } }
                                tr { td { code { "selectable" } } td { code { "bool" } } td { "Enable node selection" } }
                                tr { td { code { "draggable" } } td { code { "bool" } } td { "Enable drag and drop" } }
                                tr { td { code { "defaultExpandAll" } } td { code { "bool" } } td { "Expand all nodes initially" } }
                                tr { td { code { "onSelect" } } td { code { "(node: TreeNode) => void" } } td { "Node select callback" } }
                            }
                        }
                    }
                }
            }
        }
    }
}
