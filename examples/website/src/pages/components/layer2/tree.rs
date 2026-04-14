use crate::components::breadcrumb;
use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

pub fn render() -> VNode {
    rsx! {
        div { id: "page-component-tree", class: "hikari-page",
            ..vec![breadcrumb(&[("Home", "/"), ("Components", "/components"), ("Layer 2 — Composed", "/components/layer2/tree"), ("Tree", "")])]
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
            }
        }
    }
}
