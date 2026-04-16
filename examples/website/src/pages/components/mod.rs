pub mod layer1;
pub mod layer2;
pub mod layer3;

use crate::components::breadcrumb;
use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

pub fn render_all() -> Vec<VNode> {
    let mut pages = Vec::new();
    pages.push(render_overview());
    pages.extend(layer1::render_all());
    pages.extend(layer2::render_all());
    pages.extend(layer3::render_all());
    pages
}

pub fn render_overview() -> VNode {
    rsx! {
        div { id: "page-components", class: "hikari-page",
            ..vec![breadcrumb(&[("Home", "/"), ("Components", "")])]
            section { class: "page-section",
                h2 { class: "page-section__title", "Components" }
                p { class: "card__body",
                    "Tairitsu provides a rich set of UI components organised into three layers of complexity."
                }
            }
            section { class: "page-section",
                h3 { class: "page-section__title", "Layer 1 \u{2014} Base" }
                p { class: "card__body", "Fundamental building blocks: Button, Form, Input, Switch, and more." }
                div { class: "card-grid",
                    div { class: "card",
                        h3 { class: "card__title", "Button" }
                        p { class: "card__body", "Interactive trigger with variants, sizes, glow effects." }
                        a { href: "/components/layer1/button", class: "hi-button hi-button-secondary", "View" }
                    }
                    div { class: "card",
                        h3 { class: "card__title", "Form" }
                        p { class: "card__body", "Form container with layout and validation." }
                        a { href: "/components/layer1/form", class: "hi-button hi-button-secondary", "View" }
                    }
                    div { class: "card",
                        h3 { class: "card__title", "Number Input" }
                        p { class: "card__body", "Numeric input with step and range controls." }
                        a { href: "/components/layer1/number-input", class: "hi-button hi-button-secondary", "View" }
                    }
                    div { class: "card",
                        h3 { class: "card__title", "Search" }
                        p { class: "card__body", "Search input with icon and debounce." }
                        a { href: "/components/layer1/search", class: "hi-button hi-button-secondary", "View" }
                    }
                    div { class: "card",
                        h3 { class: "card__title", "Switch" }
                        p { class: "card__body", "Boolean toggle with sizes and labels." }
                        a { href: "/components/layer1/switch", class: "hi-button hi-button-secondary", "View" }
                    }
                    div { class: "card",
                        h3 { class: "card__title", "Feedback" }
                        p { class: "card__body", "Alerts, messages, status badges." }
                        a { href: "/components/layer1/feedback", class: "hi-button hi-button-secondary", "View" }
                    }
                    div { class: "card",
                        h3 { class: "card__title", "Display" }
                        p { class: "card__body", "Read-only data and status display." }
                        a { href: "/components/layer1/display", class: "hi-button hi-button-secondary", "View" }
                    }
                    div { class: "card",
                        h3 { class: "card__title", "Avatar" }
                        p { class: "card__body", "User avatar with sizes, colors, groups." }
                        a { href: "/components/layer1/avatar", class: "hi-button hi-button-secondary", "View" }
                    }
                    div { class: "card",
                        h3 { class: "card__title", "Image" }
                        p { class: "card__body", "Image with lazy loading and error fallback." }
                        a { href: "/components/layer1/image", class: "hi-button hi-button-secondary", "View" }
                    }
                    div { class: "card",
                        h3 { class: "card__title", "Tag" }
                        p { class: "card__body", "Labeling with color variants." }
                        a { href: "/components/layer1/tag", class: "hi-button hi-button-secondary", "View" }
                    }
                    div { class: "card",
                        h3 { class: "card__title", "Empty" }
                        p { class: "card__body", "Placeholder for empty data states." }
                        a { href: "/components/layer1/empty", class: "hi-button hi-button-secondary", "View" }
                    }
                    div { class: "card",
                        h3 { class: "card__title", "Comment" }
                        p { class: "card__body", "Comments with nested replies." }
                        a { href: "/components/layer1/comment", class: "hi-button hi-button-secondary", "View" }
                    }
                    div { class: "card",
                        h3 { class: "card__title", "Description List" }
                        p { class: "card__body", "Key-value metadata display." }
                        a { href: "/components/layer1/description-list", class: "hi-button hi-button-secondary", "View" }
                    }
                }
            }
            section { class: "page-section",
                h3 { class: "page-section__title", "Layer 2 \u{2014} Composed" }
                p { class: "card__body", "Higher-level patterns built from base primitives: Navigation, Table, Tree, etc." }
                div { class: "card-grid",
                    div { class: "card",
                        h3 { class: "card__title", "Navigation" }
                        p { class: "card__body", "Tabs, breadcrumbs, sidebar menus." }
                        a { href: "/components/layer2/navigation", class: "hi-button hi-button-secondary", "View" }
                    }
                    div { class: "card",
                        h3 { class: "card__title", "Collapsible" }
                        p { class: "card__body", "Expandable/collapsible panels, accordion." }
                        a { href: "/components/layer2/collapsible", class: "hi-button hi-button-secondary", "View" }
                    }
                    div { class: "card",
                        h3 { class: "card__title", "Data" }
                        p { class: "card__body", "Statistics cards, data lists." }
                        a { href: "/components/layer2/data", class: "hi-button hi-button-secondary", "View" }
                    }
                    div { class: "card",
                        h3 { class: "card__title", "Table" }
                        p { class: "card__body", "Data table with bordered, striped, hover." }
                        a { href: "/components/layer2/table", class: "hi-button hi-button-secondary", "View" }
                    }
                    div { class: "card",
                        h3 { class: "card__title", "Tree" }
                        p { class: "card__body", "Hierarchical tree view." }
                        a { href: "/components/layer2/tree", class: "hi-button hi-button-secondary", "View" }
                    }
                    div { class: "card",
                        h3 { class: "card__title", "Pagination" }
                        p { class: "card__body", "Page navigation with info display." }
                        a { href: "/components/layer2/pagination", class: "hi-button hi-button-secondary", "View" }
                    }
                    div { class: "card",
                        h3 { class: "card__title", "QRCode" }
                        p { class: "card__body", "QR code generation." }
                        a { href: "/components/layer2/qrcode", class: "hi-button hi-button-secondary", "View" }
                    }
                    div { class: "card",
                        h3 { class: "card__title", "Timeline" }
                        p { class: "card__body", "Sequential events with status colors." }
                        a { href: "/components/layer2/timeline", class: "hi-button hi-button-secondary", "View" }
                    }
                    div { class: "card",
                        h3 { class: "card__title", "Form (Composed)" }
                        p { class: "card__body", "Multi-step, multi-column, inline forms." }
                        a { href: "/components/layer2/form", class: "hi-button hi-button-secondary", "View" }
                    }
                    div { class: "card",
                        h3 { class: "card__title", "Cascader" }
                        p { class: "card__body", "Cascading hierarchical selection." }
                        a { href: "/components/layer2/cascader", class: "hi-button hi-button-secondary", "View" }
                    }
                    div { class: "card",
                        h3 { class: "card__title", "Transfer" }
                        p { class: "card__body", "Dual-list transfer component." }
                        a { href: "/components/layer2/transfer", class: "hi-button hi-button-secondary", "View" }
                    }
                    div { class: "card",
                        h3 { class: "card__title", "Feedback (Composed)" }
                        p { class: "card__body", "Progress bars, skeleton, result pages." }
                        a { href: "/components/layer2/feedback", class: "hi-button hi-button-secondary", "View" }
                    }
                }
            }
            section { class: "page-section",
                h3 { class: "page-section__title", "Layer 3 \u{2014} Complex" }
                p { class: "card__body", "Full-featured widgets: Media player, Editor, Visualization, User Guide." }
                div { class: "card-grid",
                    div { class: "card",
                        h3 { class: "card__title", "Media" }
                        p { class: "card__body", "Video and audio player with controls." }
                        a { href: "/components/layer3/media", class: "hi-button hi-button-secondary", "View" }
                    }
                    div { class: "card",
                        h3 { class: "card__title", "Editor" }
                        p { class: "card__body", "Rich text editor with markdown." }
                        a { href: "/components/layer3/editor", class: "hi-button hi-button-secondary", "View" }
                    }
                    div { class: "card",
                        h3 { class: "card__title", "Visualization" }
                        p { class: "card__body", "Charts, progress rings, dashboards." }
                        a { href: "/components/layer3/visualization", class: "hi-button hi-button-secondary", "View" }
                    }
                    div { class: "card",
                        h3 { class: "card__title", "User Guide" }
                        p { class: "card__body", "Interactive guided tour and onboarding." }
                        a { href: "/components/layer3/user-guide", class: "hi-button hi-button-secondary", "View" }
                    }
                    div { class: "card",
                        h3 { class: "card__title", "Zoom Controls" }
                        p { class: "card__body", "Zoom/pan for canvas, maps, diagrams." }
                        a { href: "/components/layer3/zoom-controls", class: "hi-button hi-button-secondary", "View" }
                    }
                }
            }
        }
    }
}
