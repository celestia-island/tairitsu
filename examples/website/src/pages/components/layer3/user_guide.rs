use crate::components::{breadcrumb, svg_icon};
use hikari_icons::MdiIcon;
use tairitsu_macros::rsx;
use tairitsu_vdom::{VNode, VText};

fn txt(s: &str) -> VNode {
    VNode::Text(VText::new(s))
}

pub fn render() -> VNode {
    rsx! {
        div { id: "page-component-user-guide", class: "hikari-page",
            ..vec![breadcrumb(&[("Home", "/"), ("Components", "/components"), ("Layer 3 \u{2014} Complex", "/components/layer3/user-guide"), ("User Guide", "")])]
            section { class: "page-section",
                h2 { class: "page-section__title", "User Guide" }
                p { class: "card__body",
                    "Interactive guided tour and onboarding component for walking users through features step-by-step."
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Guide Steps" }
                    div { class: "demo-block__body",
                        ol { class: "user-guide-steps",
                            li {
                                strong { "Welcome to Tairitsu" }
                                p { "This guide will walk you through the main features of the framework. Follow the steps to get started quickly." }
                            }
                            li {
                                strong { "Create a Component" }
                                p { "Use the rsx! macro to create declarative UI components. Components are plain Rust functions that return VNode." }
                            }
                            li {
                                strong { "Add Interactivity" }
                                p { "Use hooks like use_signal and use_state to add reactive state management to your components." }
                            }
                            li {
                                strong { "Style with Hikari" }
                                p { "Apply Hikari design system classes for consistent, beautiful styling across all your components." }
                            }
                            li {
                                strong { "Build and Deploy" }
                                p { "Compile to WebAssembly using the tairitsu-packager and deploy your application to any static hosting." }
                            }
                        }
                    }
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Tour Overlay (Visual)" }
                    div { class: "demo-block__body",
                         div { class: "guide-tour-box",
                            div { class: "guide-tour-box__hint", ..vec![svg_icon(MdiIcon::LightningBolt, 16, ""), txt(" Tooltip pointing to an element")] }
                            div { class: "guide-tour-box__step", "Step 2 of 5: This is the main content area where your components are rendered." }
                            div { class: "guide-tour-actions",
                                button { class: "hi-button hi-button-secondary guide-tour-btn", "Skip" }
                                button { class: "hi-button hi-button-primary guide-tour-btn", "Next" }
                            }
                            div { class: "guide-dots",
                                div { class: "guide-dot" }
                                div { class: "guide-dot guide-dot--active" }
                                div { class: "guide-dot" }
                                div { class: "guide-dot" }
                                div { class: "guide-dot" }
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
                                tr { td { code { "steps" } } td { code { "GuideStep[]" } } td { "Guide step definitions" } }
                                tr { td { code { "current" } } td { code { "number" } } td { "Current step index" } }
                                tr { td { code { "target" } } td { code { "string" } } td { "CSS selector for highlight target" } }
                                tr { td { code { "placement" } } td { code { "top | bottom | left | right" } } td { "Tooltip placement" } }
                                tr { td { code { "skippable" } } td { code { "bool" } } td { "Allow skipping the guide" } }
                                tr { td { code { "maskClosable" } } td { code { "bool" } } td { "Close on mask click" } }
                                tr { td { code { "onFinish" } } td { code { "() => void" } } td { "Guide complete callback" } }
                                tr { td { code { "onSkip" } } td { code { "() => void" } } td { "Guide skip callback" } }
                            }
                        }
                    }
                }
            }
        }
    }
}
