use crate::components::breadcrumb;
use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

pub fn render() -> VNode {
    rsx! {
        div { id: "page-component-feedback", class: "hikari-page",
            ..vec![breadcrumb(&[("Home", "/"), ("Components", "/components"), ("Layer 1 — Base", "/components/layer1/feedback"), ("Feedback", "")])]
            section { class: "page-section",
                h2 { class: "page-section__title", "Feedback" }
                p { class: "card__body",
                    "Visual feedback components: alerts, messages, notifications, and loading indicators."
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Alert Variants" }
                    div { class: "demo-block__body",
                        div { class: "hi-alert hi-alert-info", "Info: This is an informational message." }
                        div { class: "hi-alert hi-alert-success", "Success: Operation completed successfully." }
                        div { class: "hi-alert hi-alert-warning", "Warning: Please review before proceeding." }
                        div { class: "hi-alert hi-alert-error", "Error: Something went wrong." }
                    }
                }
            }
        }
    }
}
