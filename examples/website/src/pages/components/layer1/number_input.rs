use crate::components::breadcrumb;
use tairitsu_macros::rsx;
use tairitsu_vdom::{VElement, VNode};

fn el(tag: &str) -> VElement {
    VElement::new(tag)
}

fn num_input(val: &str, min: &str, max: &str, step: &str, ph: &str) -> VNode {
    VNode::Element(
        el("input")
            .attr("type", "number")
            .attr("value", val)
            .attr("min", min)
            .attr("max", max)
            .attr("step", step)
            .attr("placeholder", ph),
    )
}

pub fn render() -> VNode {
    rsx! {
        div { id: "page-component-number-input", class: "hikari-page",
            ..vec![breadcrumb(&[("Home", "/"), ("Components", "/components"), ("Layer 1 \u{2014} Base", "/components/layer1/number-input"), ("Number Input", "")])]
            section { class: "page-section",
                h2 { class: "page-section__title", "Number Input" }
                p { class: "card__body",
                    "Input field for numeric values with increment/decrement controls, step configuration, and range validation."
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Basic Number Input" }
                    div { class: "demo-block__body",
                        div { class: "form-group",
                            label { "Quantity" }
                            div { class: "form-input-wrapper", ..vec![num_input("0", "0", "100", "1", "0")] }
                        }
                    }
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Decimal Step" }
                    div { class: "demo-block__body",
                        div { class: "form-group",
                            label { "Price" }
                            div { class: "form-input-wrapper", ..vec![num_input("0.00", "0", "9999", "0.01", "0.00")] }
                        }
                    }
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Large Numbers" }
                    div { class: "demo-block__body",
                        div { class: "form-row",
                            div { class: "form-group",
                                label { "Min Value: 0" }
                                div { class: "form-input-wrapper", ..vec![num_input("0", "0", "10000", "1", "0")] }
                            }
                            div { class: "form-group",
                                label { "Min Value: -100" }
                                div { class: "form-input-wrapper", ..vec![num_input("0", "-100", "100", "5", "0")] }
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
                                tr { td { code { "min" } } td { code { "number" } } td { code { "-Infinity" } } td { "Minimum value" } }
                                tr { td { code { "max" } } td { code { "number" } } td { code { "Infinity" } } td { "Maximum value" } }
                                tr { td { code { "step" } } td { code { "number" } } td { code { "1" } } td { "Increment step" } }
                                tr { td { code { "precision" } } td { code { "number" } } td { code { "-" } } td { "Decimal precision" } }
                                tr { td { code { "disabled" } } td { code { "bool" } } td { code { "false" } } td { "Disable input" } }
                            }
                        }
                    }
                }
            }
        }
    }
}
