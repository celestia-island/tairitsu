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

fn num_input(val: &str, ph: &str) -> VNode {
    VNode::Element(el("div").class("num-input-wrap").children(vec![
            VNode::Element(
                el("input")
                    .attr("type", "number")
                    .attr("value", val)
                    .attr("placeholder", ph),
            ),
            VNode::Element(el("button").class("num-input-btn").attr("type", "button")
                .child(svg_icon(MdiIcon::Minus, 14, ""))),
            VNode::Element(el("button").class("num-input-btn").attr("type", "button")
                .child(svg_icon(MdiIcon::Plus, 14, ""))),
        ]))
}

pub fn render() -> VNode {
    rsx! {
        div { id: "page-component-number-input", class: "hikari-page",
            ..vec![breadcrumb(&[("Home", "/"), ("Components", "/components"), ("Layer 1 \u{2014} Base", "/components/layer1/number-input"), ("Number Input", "")])]
            section { class: "page-section",
                h2 { class: "page-section__title", "Number Input" }
                p { class: "page-section__description",
                    "Input field for numeric values with full-height increment/decrement buttons, step configuration, and range validation."
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Basic Number Input" }
                    div { class: "demo-block__body",
                        div { class: "form-group",
                            label { "Quantity" }
                            ..vec![num_input("0", "0")]
                        }
                    }
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Decimal Step" }
                    div { class: "demo-block__body",
                        div { class: "form-group",
                            label { "Price" }
                            ..vec![num_input("0.00", "0.00")]
                        }
                    }
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Large Numbers" }
                    div { class: "demo-block__body",
                        div { class: "form-row",
                            div { class: "form-group",
                                label { "Minimum" }
                                ..vec![num_input("0", "0")]
                            }
                            div { class: "form-group",
                                label { "Maximum (negative)" }
                                ..vec![num_input("0", "0")]
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
                                tr { td { code { "min" } } td { code { "number" } } td { "Minimum value (-Infinity for unbounded)" } }
                                tr { td { code { "max" } } td { code { "number" } } td { "Maximum value (Infinity for unbounded)" } }
                                tr { td { code { "step" } } td { code { "number" } } td { "Increment step (default: 1)" } }
                                tr { td { code { "precision" } } td { code { "number" } } td { "Decimal precision" } }
                                tr { td { code { "disabled" } } td { code { "bool" } } td { "Disable input" } }
                            }
                        }
                    }
                }
            }
        }
    }
}
