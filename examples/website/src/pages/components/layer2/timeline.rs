use crate::components::breadcrumb;
use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

pub fn render() -> VNode {
    rsx! {
        div { id: "page-component-timeline", class: "hikari-page",
            ..vec![breadcrumb(&[("Home", "/"), ("Components", "/components"), ("Layer 2 — Composed", "/components/layer2/timeline"), ("Timeline", "")])]
            section { class: "page-section",
                h2 { class: "page-section__title", "Timeline" }
                p { class: "card__body",
                    "Vertical or horizontal timeline for displaying sequential events, activity logs, or process steps."
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Basic Timeline" }
                    div { class: "demo-block__body",
                        div { class: "hi-timeline",
                            div { class: "hi-timeline-item",
                                div { class: "hi-timeline-dot" }
                                div { class: "hi-timeline-content",
                                    strong { "Project Started" }
                                    p { "Initial setup and planning phase." }
                                    span { class: "hi-timeline-time", "2024-01-01" }
                                }
                            }
                            div { class: "hi-timeline-item",
                                div { class: "hi-timeline-dot" }
                                div { class: "hi-timeline-content",
                                    strong { "Alpha Release" }
                                    p { "First public preview release." }
                                    span { class: "hi-timeline-time", "2024-03-15" }
                                }
                            }
                            div { class: "hi-timeline-item",
                                div { class: "hi-timeline-dot" }
                                div { class: "hi-timeline-content",
                                    strong { "Beta Launch" }
                                    p { "Feature-complete beta version." }
                                    span { class: "hi-timeline-time", "2024-06-01" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
