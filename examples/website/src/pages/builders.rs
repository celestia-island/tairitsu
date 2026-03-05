use tairitsu_hooks::use_signal;
use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

pub fn builders() -> VNode {
    let intensity = use_signal(0.5);

    rsx! {
        div {
            class: "page builders-demo",
            h1 {
                "Builder System Demo"
            }

            section {
                class: "demo-section",
                h2 {
                    "1. StyleBuilder"
                }
                p {
                    "Build CSS styles programmatically with type safety"
                }
                div {
                    class: "demo-box",
                    style: format!(
                        "background: linear-gradient(135deg, rgba(102, 126, 234, {}) 0%, rgba(118, 75, 162, {}) 100%); padding: 30px; border-radius: 12px; color: white;",
                        intensity.get(),
                        intensity.get()
                    ),
                    h3 {
                        "Dynamic Styling"
                    },
                    p {
                        "Adjust intensity with the slider below"
                    }
                }
                input {
                    type: "range",
                    min: "0",
                    max: "1",
                    step: "0.1",
                    value: format!("{}", intensity.get()),
                    oninput: move |e| {
                        if let Some(val) = e.target().and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok()) {
                            if let Ok(v) = val.value().parse::<f64>() {
                                intensity.set(v);
                            }
                        }
                    }
                }
                pre {
                    code {
                        r#"
// Concept: StyleBuilder integration
// (Full implementation in hikari-animation)
let style = StyleBuilder::new()
    .add(CssProperty::Width, "100px")
    .add(CssProperty::Height, "100px")
    .add_custom("--glow-intensity", "0.5")
    .build();
"#
                    }
                }
            }

            section {
                class: "demo-section",
                h2 {
                    "2. ClassesBuilder"
                }
                p {
                    "Build class names with conditional logic"
                }
                div {
                    class: "demo-box",
                    div {
                        class: "demo-class active large",
                        "Classes: demo-class active large"
                    }
                }
                pre {
                    code {
                        r#"
// Concept: ClassesBuilder integration
// (Full implementation in hikari-palette)
let is_active = true;
let is_large = true;

let classes = ClassesBuilder::new()
    .add("demo-class")
    .add_if("active", || is_active)
    .add_if("large", || is_large)
    .build();
"#
                    }
                }
            }

            section {
                class: "demo-section",
                h2 {
                    "3. AnimationBuilder"
                }
                p {
                    "Configure animations declaratively"
                }
                div {
                    class: "demo-box animated",
                    style: "animation: pulse 2s infinite;",
                    "Animated Element"
                }
                style {
                    r#"
                    @keyframes pulse {
                        0%, 100% { transform: scale(1); }
                        50% { transform: scale(1.05); }
                    }
                    "#
                }
                pre {
                    code {
                        r#"
// Concept: AnimationBuilder integration
// (Full implementation in hikari-animation)
let animation = AnimationBuilder::new()
    .duration(Duration::from_secs(2))
    .timing(TimingFunction::EaseInOut)
    .keyframes(|kf| {
        kf.add(0.0, |s| s.transform("scale(1)"))
          .add(0.5, |s| s.transform("scale(1.05)"))
          .add(1.0, |s| s.transform("scale(1)"))
    })
    .build();
"#
                    }
                }
            }
        }
    }
}
