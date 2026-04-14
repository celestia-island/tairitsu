use crate::components::breadcrumb;
use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

pub fn render() -> VNode {
    rsx! {
        div { id: "page-component-user-guide", class: "hikari-page",
            ..vec![breadcrumb(&[("Home", "/"), ("Components", "/components"), ("Layer 3 — Complex", "/components/layer3/user-guide"), ("User Guide", "")])]
            section { class: "page-section",
                h2 { class: "page-section__title", "User Guide" }
                p { class: "card__body",
                    "Interactive guided tour and onboarding component for walking users through features step-by-step."
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Guide Steps" }
                    div { class: "demo-block__body",
                        ol { class: "user-guide-steps",
                            li { strong { "Step 1: Welcome" } p { "Introduction to the interface." } }
                            li { strong { "Step 2: Navigation" } p { "Learn how to move around." } }
                            li { strong { "Step 3: Actions" } p { "Discover available actions." } }
                            li { strong { "Step 4: Complete" } p { "You're ready to go!" } }
                        }
                    }
                }
            }
        }
    }
}
