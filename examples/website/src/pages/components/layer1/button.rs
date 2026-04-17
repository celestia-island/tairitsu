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

pub fn render() -> VNode {
    rsx! {
        div { id: "page-component-button", class: "hikari-page",
            ..vec![breadcrumb(&[("Home", "/"), ("Components", "/components"), ("Layer 1 \u{2014} Base", "/components/layer1/button"), ("Button", "")])]
            section { class: "page-section",
                h2 { class: "page-section__title", "Button" }
                p { class: "page-section__description",
                    "A fundamental interactive element that triggers an action when clicked. Supports multiple variants, sizes, and states."
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Button Variants" }
                    div { class: "demo-block__body",
                        div { class: "demo-row",
                            button { class: "hi-button hi-button-primary", "Primary" }
                            button { class: "hi-button hi-button-secondary", "Secondary" }
                            button { class: "hi-button hi-button-tertiary", "Tertiary" }
                        }
                    }
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Button with Glow Effect" }
                    div { class: "demo-block__body",
                        div { class: "demo-row",
                            button { class: "hi-button hi-button-primary hi-glow-wrapper hi-glow-soft", "Primary Glow" }
                            button { class: "hi-button hi-button-secondary hi-glow-wrapper hi-glow-soft", "Secondary Glow" }
                        }
                    }
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Button Sizes" }
                    div { class: "demo-block__body",
                        div { class: "demo-row",
                            button { class: "hi-button hi-button-primary hi-button-sm", "Small" }
                            button { class: "hi-button hi-button-primary", "Default" }
                            button { class: "hi-button hi-button-primary hi-button-lg", "Large" }
                        }
                    }
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Button States" }
                    div { class: "demo-block__body",
                        div { class: "demo-row",
                            button { class: "hi-button hi-button-primary", "Normal" }
                            button { class: "hi-button hi-button-primary hi-button-disabled", "Disabled" }
                            button { class: "hi-button hi-button-primary hi-button-sm", ..vec![svg_icon(MdiIcon::Upload, 14, "")] }
                        }
                    }
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Button Group" }
                    div { class: "demo-block__body",
                        div { class: "demo-row",
                            button { class: "hi-button hi-button-secondary", "Cancel" }
                            button { class: "hi-button hi-button-primary", "Confirm" }
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
                                tr { td { code { "variant" } } td { code { "primary | secondary | tertiary" } } td { "Button visual style" } }
                                tr { td { code { "size" } } td { code { "sm | default | lg" } } td { "Button size preset" } }
                                tr { td { code { "disabled" } } td { code { "bool" } } td { "Disable the button" } }
                                tr { td { code { "glow" } } td { code { "dim | soft | bright" } } td { "Glow hover intensity" } }
                                tr { td { code { "icon" } } td { code { "bool" } } td { "Icon-only button mode" } }
                            }
                        }
                    }
                }
            }
        }
    }
}
