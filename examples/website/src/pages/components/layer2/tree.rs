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

fn arrow() -> VNode {
    svg_icon(MdiIcon::ChevronRight, 12, "tree-arrow")
}

fn folder_icon() -> VNode {
    svg_icon(MdiIcon::Package, 16, "tree-node-icon__folder")
}

fn file_icon() -> VNode {
    svg_icon(MdiIcon::TextBox, 16, "tree-node-icon__file")
}

fn tree_item(label: &str, children: Vec<VNode>) -> VNode {
    VNode::Element(el("li").children(vec![
        VNode::Element(el("span").children(vec![arrow(), txt(label)])),
        VNode::Element(el("ul").children(children)),
    ]))
}

fn tree_leaf(label: &str) -> VNode {
    VNode::Element(el("li").child(txt(label)))
}

pub fn render() -> VNode {
    rsx! {
        div { id: "page-component-tree", class: "hikari-page",
            ..vec![breadcrumb(&[("Home", "/"), ("Components", "/components"), ("Layer 2 \u{2014} Composed", "/components/layer2/tree"), ("Tree", "")])]
            section { class: "page-section",
                h2 { class: "page-section__title", "Tree" }
                p { class: "page-section__description",
                    "Hierarchical tree view component with expand/collapse, checkbox selection, drag-and-drop, and async loading."
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Basic Tree" }
                    div { class: "demo-block__body",
                        ul { class: "hi-tree",
                            ..vec![
                                tree_item(" Root", vec![
                                    tree_item(" Child 1", vec![tree_leaf("Leaf A"), tree_leaf("Leaf B")]),
                                    tree_item(" Child 2", vec![tree_leaf("Leaf C")]),
                                ]),
                            ]
                        }
                    }
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "File Tree" }
                    div { class: "demo-block__body",
                        ul { class: "hi-tree",
                            ..vec![
                                tree_item(" packages", vec![
                                    tree_item(" vdom", vec![tree_leaf("src/lib.rs"), tree_leaf("Cargo.toml")]),
                                    tree_item(" hooks", vec![tree_leaf("src/lib.rs"), tree_leaf("Cargo.toml")]),
                                    tree_item(" macros", vec![tree_leaf("src/lib.rs"), tree_leaf("Cargo.toml")]),
                                ]),
                                tree_item(" examples", vec![
                                    tree_item(" website", vec![tree_leaf("src/app.rs"), tree_leaf("Cargo.toml")]),
                                ]),
                                tree_leaf("Cargo.toml"),
                                tree_leaf("justfile"),
                            ]
                        }
                    }
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Tree with Icons" }
                    div { class: "demo-block__body",
                         ul { class: "hi-tree",
                            ..vec![VNode::Element(el("li").class("tree-node-icon")
                                .children(vec![
                                    VNode::Element(el("span").children(vec![folder_icon(), arrow(), txt(" src")])),
                                    VNode::Element(el("ul").children(vec![
                                        VNode::Element(el("li").class("tree-node-icon")
                                            .child(VNode::Element(el("span").children(vec![file_icon(), txt("lib.rs")])))),
                                        VNode::Element(el("li").class("tree-node-icon")
                                            .child(VNode::Element(el("span").children(vec![file_icon(), txt("app.rs")])))),
                                        VNode::Element(el("li").class("tree-node-icon")
                                            .children(vec![
                                                VNode::Element(el("span").children(vec![folder_icon(), arrow(), txt(" pages")])),
                                                VNode::Element(el("ul").children(vec![
                                                    VNode::Element(el("li").class("tree-node-icon")
                                                        .child(VNode::Element(el("span").children(vec![file_icon(), txt("home.rs")])))),
                                                ])),
                                            ])),
                                    ])),
                                ]))]
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
