use crate::components::breadcrumb;
use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

pub fn render() -> VNode {
    rsx! {
        div { id: "page-component-visualization", class: "hikari-page",
            ..vec![breadcrumb(&[("Home", "/"), ("Components", "/components"), ("Layer 3 — Complex", "/components/layer3/visualization"), ("Visualization", "")])]
            section { class: "page-section",
                h2 { class: "page-section__title", "Visualization" }
                p { class: "card__body",
                    "Data visualization components: charts (line, bar, pie, scatter), graphs, and interactive dashboards."
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Chart Example" }
                    div { class: "demo-block__body",
                        div { class: "chart-placeholder",
                            style: "width:100%;max-width:600px;height:300px;background:rgba(255,255,255,0.03);border-radius:8px;display:flex;align-items:center;justify-content:center;color:#666;",
                            "\u{1F4CA} Chart Visualization Area"
                        }
                    }
                }
            }
        }
    }
}
