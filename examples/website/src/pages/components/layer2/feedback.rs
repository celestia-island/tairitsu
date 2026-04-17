use crate::components::{breadcrumb, svg_icon};
use hikari_icons::MdiIcon;
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
                                div { class: "progress-label-row",
                                    span { class: "progress-label-row__text", "Upload Progress" }
                                    span { class: "progress-label-row__value progress-label-row__value--primary", "65%" }
                                }
                                div { class: "hi-progress-bar",
                                    div { class: "hi-progress-fill", style: "width:65%;" }
                                }
                            }
                            div {
                                div { class: "progress-label-row",
                                    span { class: "progress-label-row__text", "Build Complete" }
                                    span { class: "progress-label-row__value progress-label-row__value--success", "100%" }
                                }
                                div { class: "hi-progress-bar",
                                    div { class: "hi-progress-fill hi-progress-fill--success", style: "width:100%;" }
                                }
                            }
                            div {
                                div { class: "progress-label-row",
                                    span { class: "progress-label-row__text", "Storage Warning" }
                                    span { class: "progress-label-row__value progress-label-row__value--warning", "85%" }
                                }
                                div { class: "hi-progress-bar",
                                    div { class: "hi-progress-fill hi-progress-fill--warning", style: "width:85%;" }
                                }
                            }
                            div {
                                div { class: "progress-label-row",
                                    span { class: "progress-label-row__text", "Memory Critical" }
                                    span { class: "progress-label-row__value progress-label-row__value--danger", "95%" }
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
                                span { class: "progress-size-label", "Small" }
                                div { class: "hi-progress-bar hi-progress-bar--sm",
                                    div { class: "hi-progress-fill", style: "width:50%;" }
                                }
                            }
                            div {
                                span { class: "progress-size-label", "Default" }
                                div { class: "hi-progress-bar",
                                    div { class: "hi-progress-fill", style: "width:50%;" }
                                }
                            }
                            div {
                                span { class: "progress-size-label", "Large" }
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
                                div { class: "result-page__icon", ..vec![svg_icon(MdiIcon::CheckboxMarkedCircle, 48, "")] }
                                div { class: "result-page__title", "Success" }
                                div { class: "result-page__desc", "Operation completed successfully." }
                                a { href: "#", class: "hi-button hi-button-primary", "Continue" }
                            }
                            div { class: "result-page",
                                div { class: "result-page__icon", ..vec![svg_icon(MdiIcon::Close, 48, "")] }
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
