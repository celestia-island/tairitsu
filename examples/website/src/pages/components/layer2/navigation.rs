use crate::components::breadcrumb;
use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

pub fn render() -> VNode {
    rsx! {
        div { id: "page-component-navigation", class: "hikari-page",
            ..vec![breadcrumb(&[("Home", "/"), ("Components", "/components"), ("Layer 2 \u{2014} Composed", "/components/layer2/navigation"), ("Navigation", "")])]
            section { class: "page-section",
                h2 { class: "page-section__title", "Navigation" }
                p { class: "card__body",
                    "Navigation components for building menus, breadcrumbs, tabs, and page-level navigation systems."
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Tab Navigation" }
                    div { class: "demo-block__body",
                        div { class: "hi-tabs",
                            div { class: "hi-tab hi-tab-active", "Overview" }
                            div { class: "hi-tab", "API Reference" }
                            div { class: "hi-tab", "Examples" }
                            div { class: "hi-tab", "Changelog" }
                        }
                    }
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Tabs with Disabled State" }
                    div { class: "demo-block__body",
                        div { class: "hi-tabs",
                            div { class: "hi-tab hi-tab-active", "General" }
                            div { class: "hi-tab", "Security" }
                            div { class: "hi-tab hi-tab-disabled", "Advanced" }
                        }
                    }
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Breadcrumb Navigation" }
                    div { class: "demo-block__body",
                        div { style: "display:flex;flex-direction:column;gap:16px;",
                            div { style: "font-size:0.875rem;color:var(--hi-color-text-secondary);",
                                a { href: "#", style: "color:var(--hi-color-text-secondary);", "Home" }
                                " / "
                                a { href: "#", style: "color:var(--hi-color-text-secondary);", "Components" }
                                " / "
                                a { href: "#", style: "color:var(--hi-color-text-secondary);", "Layer 2" }
                                " / "
                                span { style: "color:var(--hi-color-text-primary);font-weight:500;", "Navigation" }
                            }
                            div { style: "font-size:0.875rem;color:var(--hi-color-text-secondary);",
                                a { href: "#", style: "color:var(--hi-color-text-secondary);", "Home" }
                                " > "
                                a { href: "#", style: "color:var(--hi-color-text-secondary);", "Guides" }
                                " > "
                                span { style: "color:var(--hi-color-text-primary);font-weight:500;", "Quick Start" }
                            }
                        }
                    }
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Sidebar Menu (simplified)" }
                    div { class: "demo-block__body",
                        div { style: "display:flex;flex-direction:column;gap:2px;max-width:220px;",
                            a { href: "#", style: "display:block;padding:6px 12px;border-radius:4px;font-size:0.875rem;color:var(--ts-color-primary);background:rgba(20,110,116,0.1);text-decoration:none;", "Dashboard" }
                            a { href: "#", style: "display:block;padding:6px 12px;border-radius:4px;font-size:0.875rem;color:var(--hi-color-text-secondary);text-decoration:none;", "Components" }
                            a { href: "#", style: "display:block;padding:6px 12px;border-radius:4px;font-size:0.875rem;color:var(--hi-color-text-secondary);text-decoration:none;", "Settings" }
                            a { href: "#", style: "display:block;padding:6px 12px;border-radius:4px;font-size:0.875rem;color:var(--hi-color-text-disabled);text-decoration:none;", "Disabled" }
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
                                tr { td { "Tabs" } td { code { "active" } } td { code { "number" } } td { "Active tab index" } }
                                tr { td { "Tabs" } td { code { "onChange" } } td { code { "(index: number) => void" } } td { "Tab switch callback" } }
                                tr { td { "Breadcrumb" } td { code { "items" } } td { code { "Array<{label, href}>" } } td { "Breadcrumb items" } }
                                tr { td { "Breadcrumb" } td { code { "separator" } } td { code { "string" } } td { "Separator character" } }
                            }
                        }
                    }
                }
            }
        }
    }
}
