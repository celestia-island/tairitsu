use crate::components::breadcrumb;
use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

pub fn render() -> VNode {
    rsx! {
        div { id: "page-component-feedback-2", class: "hikari-page",
            ..vec![breadcrumb(&[("Home", "/"), ("Components", "/components"), ("Layer 2 — Composed", "/components/layer2/feedback"), ("Feedback", "")])]
            section { class: "page-section",
                h2 { class: "page-section__title", "Feedback (Composed)" }
                p { class: "card__body",
                    "Advanced feedback components: progress bars, spinners, skeleton loaders, result pages."
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Progress Bar" }
                    div { class: "demo-block__body",
                        div { class: "form-group", label { "Upload Progress" }
                            div { class: "hi-progress-bar",
                                div { class: "hi-progress-fill", style: "width:65%;" }
                            }
                            span { "65%" }
                        }
                    }
                }
            }
        }
    }
}
