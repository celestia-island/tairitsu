use crate::components::breadcrumb;
use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

pub fn render() -> VNode {
    rsx! {
        div { id: "page-component-button", class: "hikari-page",
            ..vec![breadcrumb(&[("Home", "/"), ("Components", "/components"), ("Layer 1 \u{2014} Base", "/components/layer1/button"), ("Button", "")])]
            section { class: "page-section",
                h2 { class: "page-section__title", "Button" }
                p { class: "card__body",
                    "A fundamental interactive element that triggers an action when clicked. Supports multiple variants, sizes, and states."
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Button Variants" }
                    div { class: "demo-block__body",
                        div { class: "demo-row",
                            a { href: "#", class: "hi-button hi-button-primary", "Primary" }
                            a { href: "#", class: "hi-button hi-button-secondary", "Secondary" }
                            a { href: "#", class: "hi-button hi-button-tertiary", "Tertiary" }
                        }
                    }
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Button with Glow Effect" }
                    div { class: "demo-block__body",
                        div { class: "demo-row",
                            a { href: "#", class: "hi-button hi-button-primary hi-glow-wrapper hi-glow-soft", "Primary Glow" }
                            a { href: "#", class: "hi-button hi-button-secondary hi-glow-wrapper hi-glow-soft", "Secondary Glow" }
                        }
                    }
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Button Sizes" }
                    div { class: "demo-block__body",
                        div { class: "demo-row",
                            a { href: "#", class: "hi-button hi-button-primary", style: "padding:4px 12px;font-size:0.75rem;", "Small" }
                            a { href: "#", class: "hi-button hi-button-primary", "Default" }
                            a { href: "#", class: "hi-button hi-button-primary", style: "padding:10px 24px;font-size:1rem;", "Large" }
                        }
                    }
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Button States" }
                    div { class: "demo-block__body",
                        div { class: "demo-row",
                            a { href: "#", class: "hi-button hi-button-primary", "Normal" }
                            a { href: "#", class: "hi-button hi-button-primary", style: "opacity:0.5;pointer-events:none;", "Disabled" }
                            a { href: "#", class: "hi-button hi-button-primary", style: "padding-left:8px;padding-right:8px;", "\u{1F4BE}" }
                        }
                    }
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Button Group" }
                    div { class: "demo-block__body",
                        div { class: "demo-row",
                            a { href: "#", class: "hi-button hi-button-secondary", "Cancel" }
                            a { href: "#", class: "hi-button hi-button-primary", "Confirm" }
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
                                tr { td { code { "size" } } td { code { "small | default | large" } } td { "Button size preset" } }
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
