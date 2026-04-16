use crate::components::breadcrumb;
use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

pub fn render() -> VNode {
    rsx! {
        div { id: "page-component-feedback-2", class: "hikari-page",
            ..vec![breadcrumb(&[("Home", "/"), ("Components", "/components"), ("Layer 2 \u{2014} Composed", "/components/layer2/feedback"), ("Feedback", "")])]
            section { class: "page-section",
                h2 { class: "page-section__title", "Feedback (Composed)" }
                p { class: "card__body",
                    "Advanced feedback components: progress bars, spinners, skeleton loaders, result pages."
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Progress Bars" }
                    div { class: "demo-block__body",
                        div { style: "display:flex;flex-direction:column;gap:16px;",
                            div {
                                div { style: "display:flex;justify-content:space-between;margin-bottom:4px;",
                                    span { style: "font-size:0.8125rem;color:var(--hi-color-text-secondary);", "Upload Progress" }
                                    span { style: "font-size:0.8125rem;color:var(--hi-color-text-primary);font-family:var(--ts-font-mono);", "65%" }
                                }
                                div { class: "hi-progress-bar",
                                    div { class: "hi-progress-fill", style: "width:65%;" }
                                }
                            }
                            div {
                                div { style: "display:flex;justify-content:space-between;margin-bottom:4px;",
                                    span { style: "font-size:0.8125rem;color:var(--hi-color-text-secondary);", "Build Complete" }
                                    span { style: "font-size:0.8125rem;color:var(--hi-color-success);font-family:var(--ts-font-mono);", "100%" }
                                }
                                div { class: "hi-progress-bar",
                                    div { class: "hi-progress-fill hi-progress-fill--success", style: "width:100%;" }
                                }
                            }
                            div {
                                div { style: "display:flex;justify-content:space-between;margin-bottom:4px;",
                                    span { style: "font-size:0.8125rem;color:var(--hi-color-text-secondary);", "Storage Warning" }
                                    span { style: "font-size:0.8125rem;color:var(--hi-color-accent);font-family:var(--ts-font-mono);", "85%" }
                                }
                                div { class: "hi-progress-bar",
                                    div { class: "hi-progress-fill hi-progress-fill--warning", style: "width:85%;" }
                                }
                            }
                            div {
                                div { style: "display:flex;justify-content:space-between;margin-bottom:4px;",
                                    span { style: "font-size:0.8125rem;color:var(--hi-color-text-secondary);", "Memory Critical" }
                                    span { style: "font-size:0.8125rem;color:var(--hi-color-danger);font-family:var(--ts-font-mono);", "95%" }
                                }
                                div { class: "hi-progress-bar",
                                    div { class: "hi-progress-fill hi-progress-fill--danger", style: "width:95%;" }
                                }
                            }
                        }
                    }
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Progress Bar Sizes" }
                    div { class: "demo-block__body",
                        div { style: "display:flex;flex-direction:column;gap:16px;",
                            div {
                                span { style: "font-size:0.75rem;color:var(--hi-color-text-disabled);", "Small" }
                                div { class: "hi-progress-bar hi-progress-bar--sm",
                                    div { class: "hi-progress-fill", style: "width:50%;" }
                                }
                            }
                            div {
                                span { style: "font-size:0.75rem;color:var(--hi-color-text-disabled);", "Default" }
                                div { class: "hi-progress-bar",
                                    div { class: "hi-progress-fill", style: "width:50%;" }
                                }
                            }
                            div {
                                span { style: "font-size:0.75rem;color:var(--hi-color-text-disabled);", "Large" }
                                div { class: "hi-progress-bar hi-progress-bar--lg",
                                    div { class: "hi-progress-fill", style: "width:50%;" }
                                }
                            }
                        }
                    }
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Skeleton Loading" }
                    div { class: "demo-block__body",
                        div { style: "display:flex;gap:16px;align-items:flex-start;",
                            div { class: "hi-skeleton hi-skeleton-avatar" }
                            div { style: "flex:1;",
                                div { class: "hi-skeleton hi-skeleton-title" }
                                div { class: "hi-skeleton hi-skeleton-text" }
                                div { class: "hi-skeleton hi-skeleton-text" }
                                div { class: "hi-skeleton hi-skeleton-text" }
                            }
                        }
                    }
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Result Pages" }
                    div { class: "demo-block__body",
                        div { style: "display:grid;grid-template-columns:1fr 1fr;gap:16px;",
                            div { class: "result-page",
                                div { class: "result-page__icon", "\u{2705}" }
                                div { class: "result-page__title", "Success" }
                                div { class: "result-page__desc", "Operation completed successfully." }
                                a { href: "#", class: "hi-button hi-button-primary", "Continue" }
                            }
                            div { class: "result-page",
                                div { class: "result-page__icon", "\u{274C}" }
                                div { class: "result-page__title", "Error" }
                                div { class: "result-page__desc", "Something went wrong. Please try again." }
                                a { href: "#", class: "hi-button hi-button-primary", "Retry" }
                            }
                        }
                    }
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "API" }
                    div { class: "demo-block__body",
                        table { class: "api-table",
                            thead {
                                tr { th { "Component" } th { "Property" } th { "Type" } th { "Description" } }
                            }
                            tbody {
                                tr { td { "Progress" } td { code { "percent" } } td { code { "number" } } td { "Progress percentage (0-100)" } }
                                tr { td { "Progress" } td { code { "status" } } td { code { "default | success | warning | danger" } } td { "Progress status color" } }
                                tr { td { "Progress" } td { code { "size" } } td { code { "sm | default | lg" } } td { "Bar height" } }
                                tr { td { "Skeleton" } td { code { "active" } } td { code { "bool" } } td { "Enable animation" } }
                                tr { td { "Result" } td { code { "status" } } td { code { "success | error | warning | info" } } td { "Result page type" } }
                            }
                        }
                    }
                }
            }
        }
    }
}
