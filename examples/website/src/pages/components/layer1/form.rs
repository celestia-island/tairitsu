use crate::components::breadcrumb;
use tairitsu_macros::rsx;
use tairitsu_vdom::{VElement, VNode, VText};

fn el(tag: &str) -> VElement {
    VElement::new(tag)
}
fn txt(s: &str) -> VNode {
    VNode::Text(VText::new(s))
}
fn make_select_input(disabled: bool) -> VNode {
    let mut sel = el("select");
    if disabled {
        sel = sel.attr("disabled", "true");
    }
    VNode::Element(sel.children(vec![
        VNode::Element(el("option").attr("value", "").child(txt("Select an option..."))),
        VNode::Element(el("option").attr("value", "dev").child(txt("Developer"))),
        VNode::Element(el("option").attr("value", "design").child(txt("Designer"))),
        VNode::Element(el("option").attr("value", "pm").child(txt("Product Manager"))),
    ]))
}

pub fn render() -> VNode {
    let text_input = |ph: &str, dis: bool| {
        let mut inp = el("input").attr("type", "text").attr("placeholder", ph);
        if dis {
            inp = inp.attr("disabled", "true");
        }
        VNode::Element(inp)
    };
    let password_input = VNode::Element(
        el("input")
            .attr("type", "password")
            .attr("placeholder", "Enter password"),
    );
    let email_input = VNode::Element(
        el("input")
            .attr("type", "email")
            .attr("placeholder", "email@example.com"),
    );
    let textarea_input = VNode::Element(
        el("textarea")
            .attr("placeholder", "Tell us about yourself...")
            .attr("rows", "3"),
    );
    let check_agree = VNode::Element(el("input").attr("type", "checkbox").attr("checked", "true"));
    let check_news = VNode::Element(el("input").attr("type", "checkbox"));

    rsx! {
        div { id: "page-component-form", class: "hikari-page",
            ..vec![breadcrumb(&[("Home", "/"), ("Components", "/components"), ("Layer 1 \u{2014} Base", "/components/layer1/form"), ("Form", "")])]
            section { class: "page-section",
                h2 { class: "page-section__title", "Form" }
                p { class: "page-section__description",
                    "Form container for collecting and validating user input. Provides layout structure and validation integration."
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Basic Form Layout" }
                    div { class: "demo-block__body",
                        div { class: "form-row",
                            div { class: "form-group",
                                label { "Username" }
                                div { class: "form-input-wrapper", ..vec![text_input("Enter username", false)] }
                            }
                            div { class: "form-group",
                                label { "Email" }
                                div { class: "form-input-wrapper", ..vec![email_input] }
                            }
                        }
                        div { class: "form-row",
                            div { class: "form-group",
                                label { "Password" }
                                div { class: "form-input-wrapper", ..vec![password_input] }
                            }
                            div { class: "form-group",
                                label { "Role" }
                                div { class: "form-input-wrapper", ..vec![make_select_input(false)] }
                            }
                        }
                        div { class: "form-group",
                            label { "Bio" }
                            div { class: "form-input-wrapper", ..vec![textarea_input] }
                        }
                        div { class: "hi-checkbox-row",
                            ..vec![check_agree]
                            span { class: "hi-checkbox-label", "I agree to the terms and conditions" }
                        }
                        div { class: "hi-checkbox-row",
                            ..vec![check_news]
                            span { class: "hi-checkbox-label", "Subscribe to newsletter" }
                        }
                        div { class: "form-actions",
                            button { class: "hi-button hi-button-secondary", "Cancel" }
                            button { class: "hi-button hi-button-primary", "Submit" }
                        }
                    }
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Disabled Form" }
                    div { class: "demo-block__body",
                        div { class: "form-row",
                            div { class: "form-group",
                                label { "Disabled Input" }
                                div { class: "form-input-wrapper", ..vec![text_input("Cannot edit", true)] }
                            }
                            div { class: "form-group",
                                label { "Disabled Select" }
                                div { class: "form-input-wrapper", ..vec![make_select_input(true)] }
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
                                tr { td { code { "label" } } td { code { "string" } } td { "Field label text" } }
                                tr { td { code { "required" } } td { code { "bool" } } td { "Mark field as required" } }
                                tr { td { code { "disabled" } } td { code { "bool" } } td { "Disable the field" } }
                                tr { td { code { "layout" } } td { code { "vertical | horizontal" } } td { "Label-input alignment" } }
                            }
                        }
                    }
                }
            }
        }
    }
}
