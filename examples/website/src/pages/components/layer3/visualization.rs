use crate::components::breadcrumb;
use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

pub fn render() -> VNode {
    rsx! {
        div { id: "page-component-visualization", class: "hikari-page",
            ..vec![breadcrumb(&[("Home", "/"), ("Components", "/components"), ("Layer 3 \u{2014} Complex", "/components/layer3/visualization"), ("Visualization", "")])]
            section { class: "page-section",
                h2 { class: "page-section__title", "Visualization" }
                p { class: "card__body",
                    "Data visualization components: charts (line, bar, pie, scatter), graphs, and interactive dashboards."
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Bar Chart" }
                    div { class: "demo-block__body",
                         div { class: "viz-chart-container",
                            div { class: "viz-bar-col",
                                div { class: "viz-bar-fill", style: "height:120px;" }
                                span { class: "viz-bar-label", "Jan" }
                            }
                            div { class: "viz-bar-col",
                                div { class: "viz-bar-fill", style: "height:80px;" }
                                span { class: "viz-bar-label", "Feb" }
                            }
                            div { class: "viz-bar-col",
                                div { class: "viz-bar-fill", style: "height:150px;" }
                                span { class: "viz-bar-label", "Mar" }
                            }
                            div { class: "viz-bar-col",
                                div { class: "viz-bar-fill", style: "height:100px;" }
                                span { class: "viz-bar-label", "Apr" }
                            }
                            div { class: "viz-bar-col",
                                div { class: "viz-bar-fill", style: "height:160px;" }
                                span { class: "viz-bar-label", "May" }
                            }
                            div { class: "viz-bar-col",
                                div { class: "viz-bar-fill", style: "height:130px;" }
                                span { class: "viz-bar-label", "Jun" }
                            }
                        }
                    }
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Progress Ring" }
                    div { class: "demo-block__body",
                         div { class: "demo-row",
                            div { class: "viz-ring", style: "background:conic-gradient(var(--ts-color-primary) 0% 75%, rgba(255,255,255,0.08) 75% 100%);",
                                div { class: "viz-ring-inner", "75%" }
                            }
                            div { class: "viz-ring", style: "background:conic-gradient(var(--hi-color-success) 0% 100%, rgba(255,255,255,0.08) 100% 100%);",
                                div { class: "viz-ring-inner", "100%" }
                            }
                            div { class: "viz-ring", style: "background:conic-gradient(var(--hi-color-accent) 0% 45%, rgba(255,255,255,0.08) 45% 100%);",
                                div { class: "viz-ring-inner", "45%" }
                            }
                        }
                    }
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Mini Dashboard" }
                    div { class: "demo-block__body",
                         div { class: "viz-dashboard-grid",
                            div { class: "viz-dashboard-card",
                                div { class: "viz-dashboard-label", "Requests / min" }
                                div { class: "viz-dashboard-value", "1,234" }
                                div { class: "viz-sparkline",
                                    div { class: "viz-sparkline-bar viz-sparkline-bar--primary", style: "height:60%;" }
                                    div { class: "viz-sparkline-bar viz-sparkline-bar--primary", style: "height:80%;" }
                                    div { class: "viz-sparkline-bar viz-sparkline-bar--primary", style: "height:45%;" }
                                    div { class: "viz-sparkline-bar viz-sparkline-bar--primary", style: "height:90%;" }
                                    div { class: "viz-sparkline-bar viz-sparkline-bar--primary", style: "height:70%;" }
                                    div { class: "viz-sparkline-bar viz-sparkline-bar--primary", style: "height:95%;" }
                                    div { class: "viz-sparkline-bar viz-sparkline-bar--primary", style: "height:85%;" }
                                }
                            }
                            div { class: "viz-dashboard-card",
                                div { class: "viz-dashboard-label", "Error Rate" }
                                div { class: "viz-dashboard-value", style: "color:var(--hi-color-success);", "0.12%" }
                                div { class: "viz-sparkline",
                                    div { class: "viz-sparkline-bar viz-sparkline-bar--success", style: "height:20%;" }
                                    div { class: "viz-sparkline-bar viz-sparkline-bar--success", style: "height:15%;" }
                                    div { class: "viz-sparkline-bar viz-sparkline-bar--success", style: "height:25%;" }
                                    div { class: "viz-sparkline-bar viz-sparkline-bar--success", style: "height:10%;" }
                                    div { class: "viz-sparkline-bar viz-sparkline-bar--success", style: "height:18%;" }
                                    div { class: "viz-sparkline-bar viz-sparkline-bar--success", style: "height:12%;" }
                                    div { class: "viz-sparkline-bar viz-sparkline-bar--success", style: "height:8%;" }
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
                                tr { td { code { "type" } } td { code { "line | bar | pie | scatter | area" } } td { "Chart type" } }
                                tr { td { code { "data" } } td { code { "DataPoint[]" } } td { "Chart data points" } }
                                tr { td { code { "xAxis" } } td { code { "AxisConfig" } } td { "X-axis configuration" } }
                                tr { td { code { "yAxis" } } td { code { "AxisConfig" } } td { "Y-axis configuration" } }
                                tr { td { code { "responsive" } } td { code { "bool" } } td { "Auto-resize on container change" } }
                                tr { td { code { "animated" } } td { code { "bool" } } td { "Enable chart animations" } }
                            }
                        }
                    }
                }
            }
        }
    }
}
