use crate::components::breadcrumb;
use tairitsu_macros::rsx;
use tairitsu_vdom::{VElement, VNode, VText};

fn el(tag: &str) -> VElement {
    VElement::new(tag)
}
fn txt(s: &str) -> VNode {
    VNode::Text(VText::new(s))
}
fn text_input(ph: &str) -> VNode {
    VNode::Element(el("input").attr("type", "text").attr("placeholder", ph))
}

pub fn render() -> VNode {
    rsx! {
        div { id: "page-component-form-2", class: "hikari-page",
            ..vec![breadcrumb(&[("Home", "/"), ("Components", "/components"), ("Layer 2 \u{2014} Composed", "/components/layer2/form"), ("Form", "")])]
            section { class: "page-section",
                h2 { class: "page-section__title", "Form (Composed)" }
                p { class: "card__body",
                    "Advanced form patterns: multi-step forms, form layouts with grid, dynamic field arrays, and complex validation."
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Multi-column Form Layout" }
                    div { class: "demo-block__body",
                        div { class: "form-row",
                            div { class: "form-group",
                                label { "First Name" }
                                div { class: "form-input-wrapper", ..vec![text_input("First name")] }
                            }
                            div { class: "form-group",
                                label { "Last Name" }
                                div { class: "form-input-wrapper", ..vec![text_input("Last name")] }
                            }
                        }
                        div { class: "form-row",
                            div { class: "form-group",
                                label { "Email" }
                                div { class: "form-input-wrapper", ..vec![VNode::Element(el("input").attr("type","email").attr("placeholder","Email address"))] }
                            }
                            div { class: "form-group",
                                label { "Phone" }
                                div { class: "form-input-wrapper", ..vec![VNode::Element(el("input").attr("type","tel").attr("placeholder","Phone number"))] }
                            }
                        }
                        div { class: "form-actions",
                         button { class: "hi-button hi-button-secondary", "Cancel" }
                             button { class: "hi-button hi-button-primary", "Save" }
                        }
                    }
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Multi-step Form" }
                    div { class: "demo-block__body",
                        div { style: "margin-bottom:16px;",
                            div { class: "step-indicator",
                                span { class: "step-circle step-circle--active", "1" }
                                span { class: "step-label step-label--active", "Account" }
                                span { class: "step-separator", "\u{2014}" }
                                span { class: "step-circle step-circle--inactive", "2" }
                                span { class: "step-label step-label--inactive", "Profile" }
                                span { class: "step-separator", "\u{2014}" }
                                span { class: "step-circle step-circle--inactive", "3" }
                                span { class: "step-label step-label--inactive", "Review" }
                            }
                            div { class: "step-progress-track",
                                div { class: "step-progress-fill", style: "width:33%;" }
                            }
                        }
                        div { class: "form-row",
                            div { class: "form-group",
                                label { "Username" }
                                div { class: "form-input-wrapper", ..vec![text_input("Choose a username")] }
                            }
                            div { class: "form-group",
                                label { "Password" }
                                div { class: "form-input-wrapper", ..vec![VNode::Element(el("input").attr("type","password").attr("placeholder","Create password"))] }
                            }
                        }
                        div { class: "form-actions",
                             button { class: "hi-button hi-button-secondary", "Back" }
                             button { class: "hi-button hi-button-primary", "Next Step" }
                        }
                    }
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Inline Form" }
                    div { class: "demo-block__body",
                        div { style: "display:flex;align-items:flex-end;gap:12px;flex-wrap:wrap;",
                            div { class: "form-group", style: "margin-bottom:0;",
                                label { "Search" }
                                div { class: "form-input-wrapper", ..vec![text_input("Search...")] }
                            }
                            div { class: "form-group", style: "margin-bottom:0;",
                                label { "Category" }
                                div { class: "form-input-wrapper",
                                    ..vec![VNode::Element(el("select").children(vec![
                                        VNode::Element(el("option").child(txt("All"))),
                                        VNode::Element(el("option").child(txt("Components"))),
                                        VNode::Element(el("option").child(txt("Guides"))),
                                    ]))]
                                }
                            }
                             button { class: "hi-button hi-button-primary", style: "margin-bottom:0;", "Search" }
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
                                tr { td { code { "layout" } } td { code { "vertical | horizontal | inline" } } td { "Form layout mode" } }
                                tr { td { code { "columns" } } td { code { "number" } } td { "Grid columns (multi-column)" } }
                                tr { td { code { "steps" } } td { code { "Step[]" } } td { "Step definitions (multi-step)" } }
                                tr { td { code { "validation" } } td { code { "ValidationRule[]" } } td { "Validation rules" } }
                                tr { td { code { "onSubmit" } } td { code { "(values) => void" } } td { "Form submit callback" } }
                            }
                        }
                    }
                }
            }
        }
    }
}
