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
                            a { href: "#", class: "hi-button hi-button-secondary", "Cancel" }
                            a { href: "#", class: "hi-button hi-button-primary", "Save" }
                        }
                    }
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Multi-step Form" }
                    div { class: "demo-block__body",
                        div { style: "margin-bottom:16px;",
                            div { style: "display:flex;align-items:center;gap:8px;margin-bottom:8px;",
                                span { style: "width:24px;height:24px;border-radius:50%;background:var(--ts-color-primary);color:#fff;display:flex;align-items:center;justify-content:center;font-size:0.75rem;font-weight:600;", "1" }
                                span { style: "font-size:0.875rem;color:var(--hi-color-text-primary);font-weight:500;", "Account" }
                                span { style: "color:var(--hi-color-text-disabled);margin:0 4px;", "\u{2014}" }
                                span { style: "width:24px;height:24px;border-radius:50%;background:rgba(255,255,255,0.1);color:var(--hi-color-text-disabled);display:flex;align-items:center;justify-content:center;font-size:0.75rem;", "2" }
                                span { style: "font-size:0.875rem;color:var(--hi-color-text-disabled);", "Profile" }
                                span { style: "color:var(--hi-color-text-disabled);margin:0 4px;", "\u{2014}" }
                                span { style: "width:24px;height:24px;border-radius:50%;background:rgba(255,255,255,0.1);color:var(--hi-color-text-disabled);display:flex;align-items:center;justify-content:center;font-size:0.75rem;", "3" }
                                span { style: "font-size:0.875rem;color:var(--hi-color-text-disabled);", "Review" }
                            }
                            div { style: "height:3px;background:rgba(255,255,255,0.08);border-radius:2px;overflow:hidden;",
                                div { style: "width:33%;height:100%;background:var(--ts-color-primary);border-radius:2px;" }
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
                            a { href: "#", class: "hi-button hi-button-secondary", "Back" }
                            a { href: "#", class: "hi-button hi-button-primary", "Next Step" }
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
                            a { href: "#", class: "hi-button hi-button-primary", style: "margin-bottom:0;", "Search" }
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
