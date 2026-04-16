use crate::components::breadcrumb;
use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

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
                        div { style: "background:rgba(255,255,255,0.03);border:1px solid var(--hi-color-border);border-radius:8px;padding:20px;text-align:center;position:relative;",
                            div { style: "font-size:0.875rem;color:var(--hi-color-text-primary);margin-bottom:8px;", "\u{1F4A1} Tooltip pointing to an element" }
                            div { style: "font-size:0.8125rem;color:var(--hi-color-text-secondary);margin-bottom:16px;", "Step 2 of 5: This is the main content area where your components are rendered." }
                            div { style: "display:flex;align-items:center;justify-content:center;gap:12px;",
                                a { href: "#", class: "hi-button hi-button-secondary", style: "padding:4px 16px;font-size:0.8125rem;", "Skip" }
                                a { href: "#", class: "hi-button hi-button-primary", style: "padding:4px 16px;font-size:0.8125rem;", "Next" }
                            }
                            div { style: "display:flex;justify-content:center;gap:4px;margin-top:12px;",
                                div { style: "width:8px;height:8px;border-radius:50%;background:var(--hi-color-text-disabled);" }
                                div { style: "width:8px;height:8px;border-radius:50%;background:var(--ts-color-primary);" }
                                div { style: "width:8px;height:8px;border-radius:50%;background:var(--hi-color-text-disabled);" }
                                div { style: "width:8px;height:8px;border-radius:50%;background:var(--hi-color-text-disabled);" }
                                div { style: "width:8px;height:8px;border-radius:50%;background:var(--hi-color-text-disabled);" }
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
