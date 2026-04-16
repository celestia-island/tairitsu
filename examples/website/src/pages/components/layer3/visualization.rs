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
                        div { style: "width:100%;max-width:500px;height:200px;background:rgba(255,255,255,0.03);border:1px solid var(--hi-color-border);border-radius:8px;padding:16px;display:flex;align-items:flex-end;gap:12px;",
                            div { style: "flex:1;display:flex;flex-direction:column;align-items:center;gap:4px;",
                                div { style: "width:100%;height:120px;background:var(--ts-color-primary);border-radius:4px 4px 0 0;opacity:0.8;" }
                                span { style: "font-size:0.6875rem;color:var(--hi-color-text-disabled);", "Jan" }
                            }
                            div { style: "flex:1;display:flex;flex-direction:column;align-items:center;gap:4px;",
                                div { style: "width:100%;height:80px;background:var(--ts-color-primary);border-radius:4px 4px 0 0;opacity:0.8;" }
                                span { style: "font-size:0.6875rem;color:var(--hi-color-text-disabled);", "Feb" }
                            }
                            div { style: "flex:1;display:flex;flex-direction:column;align-items:center;gap:4px;",
                                div { style: "width:100%;height:150px;background:var(--ts-color-primary);border-radius:4px 4px 0 0;opacity:0.8;" }
                                span { style: "font-size:0.6875rem;color:var(--hi-color-text-disabled);", "Mar" }
                            }
                            div { style: "flex:1;display:flex;flex-direction:column;align-items:center;gap:4px;",
                                div { style: "width:100%;height:100px;background:var(--ts-color-primary);border-radius:4px 4px 0 0;opacity:0.8;" }
                                span { style: "font-size:0.6875rem;color:var(--hi-color-text-disabled);", "Apr" }
                            }
                            div { style: "flex:1;display:flex;flex-direction:column;align-items:center;gap:4px;",
                                div { style: "width:100%;height:160px;background:var(--ts-color-primary);border-radius:4px 4px 0 0;opacity:0.8;" }
                                span { style: "font-size:0.6875rem;color:var(--hi-color-text-disabled);", "May" }
                            }
                            div { style: "flex:1;display:flex;flex-direction:column;align-items:center;gap:4px;",
                                div { style: "width:100%;height:130px;background:var(--ts-color-primary);border-radius:4px 4px 0 0;opacity:0.8;" }
                                span { style: "font-size:0.6875rem;color:var(--hi-color-text-disabled);", "Jun" }
                            }
                        }
                    }
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Progress Ring" }
                    div { class: "demo-block__body",
                        div { class: "demo-row",
                            div { style: "width:80px;height:80px;border-radius:50%;background:conic-gradient(var(--ts-color-primary) 0% 75%, rgba(255,255,255,0.08) 75% 100%);display:flex;align-items:center;justify-content:center;",
                                div { style: "width:60px;height:60px;border-radius:50%;background:var(--hi-color-surface);display:flex;align-items:center;justify-content:center;font-size:0.875rem;font-weight:600;color:var(--hi-color-text-primary);", "75%" }
                            }
                            div { style: "width:80px;height:80px;border-radius:50%;background:conic-gradient(var(--hi-color-success) 0% 100%, rgba(255,255,255,0.08) 100% 100%);display:flex;align-items:center;justify-content:center;",
                                div { style: "width:60px;height:60px;border-radius:50%;background:var(--hi-color-surface);display:flex;align-items:center;justify-content:center;font-size:0.875rem;font-weight:600;color:var(--hi-color-text-primary);", "100%" }
                            }
                            div { style: "width:80px;height:80px;border-radius:50%;background:conic-gradient(var(--hi-color-accent) 0% 45%, rgba(255,255,255,0.08) 45% 100%);display:flex;align-items:center;justify-content:center;",
                                div { style: "width:60px;height:60px;border-radius:50%;background:var(--hi-color-surface);display:flex;align-items:center;justify-content:center;font-size:0.875rem;font-weight:600;color:var(--hi-color-text-primary);", "45%" }
                            }
                        }
                    }
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Mini Dashboard" }
                    div { class: "demo-block__body",
                        div { style: "display:grid;grid-template-columns:1fr 1fr;gap:12px;",
                            div { style: "background:rgba(255,255,255,0.03);border:1px solid var(--hi-color-border);border-radius:8px;padding:16px;",
                                div { style: "font-size:0.8125rem;color:var(--hi-color-text-secondary);margin-bottom:4px;", "Requests / min" }
                                div { style: "font-size:1.5rem;font-weight:700;color:var(--hi-color-text-primary);", "1,234" }
                                div { style: "height:40px;display:flex;align-items:flex-end;gap:2px;margin-top:8px;",
                                    div { style: "flex:1;height:60%;background:var(--ts-color-primary);opacity:0.5;border-radius:2px 2px 0 0;" }
                                    div { style: "flex:1;height:80%;background:var(--ts-color-primary);opacity:0.5;border-radius:2px 2px 0 0;" }
                                    div { style: "flex:1;height:45%;background:var(--ts-color-primary);opacity:0.5;border-radius:2px 2px 0 0;" }
                                    div { style: "flex:1;height:90%;background:var(--ts-color-primary);opacity:0.5;border-radius:2px 2px 0 0;" }
                                    div { style: "flex:1;height:70%;background:var(--ts-color-primary);opacity:0.5;border-radius:2px 2px 0 0;" }
                                    div { style: "flex:1;height:95%;background:var(--ts-color-primary);opacity:0.5;border-radius:2px 2px 0 0;" }
                                    div { style: "flex:1;height:85%;background:var(--ts-color-primary);opacity:0.5;border-radius:2px 2px 0 0;" }
                                }
                            }
                            div { style: "background:rgba(255,255,255,0.03);border:1px solid var(--hi-color-border);border-radius:8px;padding:16px;",
                                div { style: "font-size:0.8125rem;color:var(--hi-color-text-secondary);margin-bottom:4px;", "Error Rate" }
                                div { style: "font-size:1.5rem;font-weight:700;color:var(--hi-color-success);", "0.12%" }
                                div { style: "height:40px;display:flex;align-items:flex-end;gap:2px;margin-top:8px;",
                                    div { style: "flex:1;height:20%;background:var(--hi-color-success);opacity:0.5;border-radius:2px 2px 0 0;" }
                                    div { style: "flex:1;height:15%;background:var(--hi-color-success);opacity:0.5;border-radius:2px 2px 0 0;" }
                                    div { style: "flex:1;height:25%;background:var(--hi-color-success);opacity:0.5;border-radius:2px 2px 0 0;" }
                                    div { style: "flex:1;height:10%;background:var(--hi-color-success);opacity:0.5;border-radius:2px 2px 0 0;" }
                                    div { style: "flex:1;height:18%;background:var(--hi-color-success);opacity:0.5;border-radius:2px 2px 0 0;" }
                                    div { style: "flex:1;height:12%;background:var(--hi-color-success);opacity:0.5;border-radius:2px 2px 0 0;" }
                                    div { style: "flex:1;height:8%;background:var(--hi-color-success);opacity:0.5;border-radius:2px 2px 0 0;" }
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
