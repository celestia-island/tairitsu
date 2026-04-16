use crate::components::breadcrumb;
use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

pub fn render() -> VNode {
    rsx! {
        div { id: "page-component-timeline", class: "hikari-page",
            ..vec![breadcrumb(&[("Home", "/"), ("Components", "/components"), ("Layer 2 \u{2014} Composed", "/components/layer2/timeline"), ("Timeline", "")])]
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
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Timeline with Status Colors" }
                    div { class: "demo-block__body",
                        div { class: "hi-timeline",
                            div { class: "hi-timeline-item hi-timeline-item--success",
                                div { class: "hi-timeline-dot" }
                                div { class: "hi-timeline-content",
                                    strong { "Build Passed" }
                                    p { "All 42 tests passed successfully." }
                                    span { class: "hi-timeline-time", "10:23 AM" }
                                }
                            }
                            div { class: "hi-timeline-item hi-timeline-item--success",
                                div { class: "hi-timeline-dot" }
                                div { class: "hi-timeline-content",
                                    strong { "Deployed to Staging" }
                                    p { "Application deployed to staging environment." }
                                    span { class: "hi-timeline-time", "10:25 AM" }
                                }
                            }
                            div { class: "hi-timeline-item hi-timeline-item--warning",
                                div { class: "hi-timeline-dot" }
                                div { class: "hi-timeline-content",
                                    strong { "Performance Warning" }
                                    p { "API response time exceeded 200ms threshold." }
                                    span { class: "hi-timeline-time", "10:30 AM" }
                                }
                            }
                            div { class: "hi-timeline-item hi-timeline-item--error",
                                div { class: "hi-timeline-dot" }
                                div { class: "hi-timeline-content",
                                    strong { "Deploy Failed" }
                                    p { "Production deployment failed due to missing config." }
                                    span { class: "hi-timeline-time", "10:35 AM" }
                                }
                            }
                        }
                    }
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Timeline with Tags" }
                    div { class: "demo-block__body",
                        div { class: "hi-timeline",
                            div { class: "hi-timeline-item",
                                div { class: "hi-timeline-dot" }
                                div { class: "hi-timeline-content",
                                    div { style: "display:flex;align-items:center;gap:8px;",
                                        strong { "v0.1.0 Released" }
                                        span { class: "hi-tag hi-tag-primary", "Milestone" }
                                    }
                                    p { "Initial public release with core vdom and hooks." }
                                    span { class: "hi-timeline-time", "2024-01-15" }
                                }
                            }
                            div { class: "hi-timeline-item",
                                div { class: "hi-timeline-dot" }
                                div { class: "hi-timeline-content",
                                    div { style: "display:flex;align-items:center;gap:8px;",
                                        strong { "v0.2.0 Released" }
                                        span { class: "hi-tag hi-tag-success", "Stable" }
                                    }
                                    p { "Added macros, RSX support, and documentation site." }
                                    span { class: "hi-timeline-time", "2024-04-01" }
                                }
                            }
                            div { class: "hi-timeline-item",
                                div { class: "hi-timeline-dot" }
                                div { class: "hi-timeline-content",
                                    div { style: "display:flex;align-items:center;gap:8px;",
                                        strong { "v0.3.0 In Progress" }
                                        span { class: "hi-tag hi-tag-warning", "Beta" }
                                    }
                                    p { "WASI component model support and packager improvements." }
                                    span { class: "hi-timeline-time", "2024-07-01" }
                                }
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
                                tr { td { code { "items" } } td { code { "TimelineItem[]" } } td { "Timeline data items" } }
                                tr { td { code { "mode" } } td { code { "left | right | alternate" } } td { "Content placement" } }
                                tr { td { code { "pending" } } td { code { "bool" } } td { "Show pending (last) item" } }
                                tr { td { code { "color" } } td { code { "string" } } td { "Dot color" } }
                                tr { td { code { "onClick" } } td { code { "(item) => void" } } td { "Item click callback" } }
                            }
                        }
                    }
                }
            }
        }
    }
}
