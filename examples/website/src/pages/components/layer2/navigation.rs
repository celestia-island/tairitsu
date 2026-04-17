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
                        div { class: "breadcrumb-nav",
                            div { class: "breadcrumb-nav__item",
                                a { href: "#", "Home" }
                                span { class: "breadcrumb-nav__separator", "/" }
                                a { href: "#", "Components" }
                                span { class: "breadcrumb-nav__separator", "/" }
                                a { href: "#", "Layer 2" }
                                span { class: "breadcrumb-nav__separator", "/" }
                                span { class: "breadcrumb-nav__current", "Navigation" }
                            }
                            div { class: "breadcrumb-nav__item",
                                a { href: "#", "Home" }
                                span { class: "breadcrumb-nav__separator", ">" }
                                a { href: "#", "Guides" }
                                span { class: "breadcrumb-nav__separator", ">" }
                                span { class: "breadcrumb-nav__current", "Quick Start" }
                            }
                        }
                    }
                }
                div { class: "demo-block",
                    h3 { class: "demo-block__title", "Sidebar Menu (simplified)" }
                    div { class: "demo-block__body",
                        div { class: "sidebar-menu",
                            a { href: "#", class: "sidebar-menu__link sidebar-menu__link--active", "Dashboard" }
                            a { href: "#", class: "sidebar-menu__link sidebar-menu__link--default", "Components" }
                            a { href: "#", class: "sidebar-menu__link sidebar-menu__link--default", "Settings" }
                            a { href: "#", class: "sidebar-menu__link sidebar-menu__link--disabled", "Disabled" }
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
