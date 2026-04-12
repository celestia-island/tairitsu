//! Shared layout components: top nav, sidebar, and aside footer.
//!
//! Uses hikari-icons (SVG-based MDI icons) matching hikari-legacy's
//! component library icon system, rendered under tairitsu dark theme.

use tairitsu_macros::rsx;
use tairitsu_vdom::{VElement, VNode, VText};

use hikari_icons::{get, MdiIcon};

fn txt(s: &str) -> VNode {
    VNode::Text(VText::new(s))
}
fn el(tag: &str) -> VElement {
    VElement::new(tag)
}

/// Render an MDI SVG icon as a VNode using VElement builder.
/// Builds the SVG as real VNode children so the diff/patch system
/// can properly create and update DOM nodes (inner_html is SSR-only).
fn svg_icon(icon: MdiIcon, size: u32, class: &str) -> VNode {
    let name = icon.to_string();
    let (view_box, path_d) = match get(&name) {
        Some(data) => (
            data.view_box.as_deref().unwrap_or("0 0 24 24"),
            data.path.as_deref().unwrap_or(""),
        ),
        None => ("0 0 24 24", "M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-2h2v2zm0-4h-2V7h2v6z"),
    };
    let full_class = if class.is_empty() {
        "hikari-icon".to_string()
    } else {
        format!("hikari-icon {}", class)
    };
    VNode::Element(
        el("div")
            .class(full_class.as_str())
            .attr("style", &format!("width:{}px;height:{}px;", size, size))
            .child(VNode::Element(
                el("svg")
                    .attr("xmlns", "http://www.w3.org/2000/svg")
                    .attr("viewBox", view_box)
                    .child(VNode::Element(
                        el("path").attr("fill", "currentColor").attr("d", path_d),
                    )),
            )),
    )
}

fn sidebar_icon(icon: MdiIcon) -> VNode {
    svg_icon(icon, 16, "hi-sidebar-icon")
}

fn arrow_icon() -> VNode {
    svg_icon(MdiIcon::ChevronRight, 10, "hi-sidebar-arrow")
}

fn submenu_title(icon: MdiIcon, label: &str) -> VNode {
    VNode::Element(
        el("div")
            .class("hi-menu-item-wrapper hi-menu-height-compact")
            .child(VNode::Element(
                el("div")
                    .class("hi-glow-wrapper hi-glow-blur-medium hi-glow-subtle")
                    .attr("style", "--glow-x:50%;--glow-y:50%;--glow-intensity:0.8")
                    .child(VNode::Element(
                        el("div")
                            .class("hi-menu-height-compact hi-menu-submenu-title")
                            .child(sidebar_icon(icon))
                            .child(arrow_icon())
                            .child(VNode::Element(
                                el("span").class("hi-menu-item-content").child(txt(label)),
                            )),
                    )),
            )),
    )
}

fn submenu_list(items: &[(&str, &str, MdiIcon)], open: bool) -> VNode {
    let children: Vec<VNode> = items
        .iter()
        .map(|(label, href, icon)| menu_item(*icon, label, href))
        .collect();
    let mut list = el("ul").attr("role", "menu").class("hi-menu-submenu-list");
    if open {
        list = list.class("hi-menu-submenu-list-open");
        list = list.attr("style", "display:block;opacity:1;transform:translateX(0)");
    }
    VNode::Element(list.children(children))
}

fn submenu_section(
    key: &str,
    icon: MdiIcon,
    label: &str,
    items: &[(&str, &str, MdiIcon)],
    open: bool,
) -> VNode {
    let mut li = el("li")
        .attr("role", "none")
        .class("hi-menu-submenu")
        .attr("data_key", key);
    if open {
        li = li.class("hi-menu-submenu-list-open");
    }
    let li_el = li
        .child(submenu_title(icon, label))
        .child(submenu_list(items, open));
    VNode::Element(
        el("ul")
            .class("hi-menu hi-menu-vertical hi-menu-compact")
            .child(VNode::Element(li_el)),
    )
}

// ============================================================
// Top Navigation (Header)
// ============================================================

pub fn top_nav() -> VNode {
    rsx! {
        header { class: "hi-header hi-header-sticky hi-header-md",
            div { class: "hi-header-left",
                button { class: "hi-header-toggle", id: "drawer-toggle",
                    svg { xmlns: "http://www.w3.org/2000/svg", fill: "none", viewBox: "0 0 24 24",
                        stroke: "currentColor", stroke_width: "2", stroke_linecap: "round", stroke_linejoin: "round",
                        path { d: "M4 6h16M4 12h16M4 18h16" }
                    }
                }
                a { href: "/", class: "hi-header-brand",
                    span { class: "hi-logo", "\u{273F}" }
                    span { style: "font-weight:700;font-size:1.15rem;margin-left:8px;", "Tairitsu" }
                }
            }
            div { class: "hi-header-right",
                nav { class: "hi-header-nav",
                    a { href: "/guides", class: "hi-header-link", "Guides" }
                    a { href: "/system", class: "hi-header-link", "System" }
                    a { href: "/packages", class: "hi-header-link", "Packages" }
                }
            }
        }
    }
}

// ============================================================
// Sidebar Navigation — SVG icons via hikari-icons
// ============================================================

pub fn sidebar() -> VNode {
    let home = rsx! {
        ul { class: "hi-menu hi-menu-vertical hi-menu-compact",
            li { role: "menuitem", class: "hi-menu-item hi-menu-item--home hi-menu-height-compact",
                data_key: "Home",
                a { href: "/",
                    ..vec![
                        sidebar_icon(MdiIcon::Home),
                        VNode::Element(el("span").class("hi-sidebar-label").child(txt("Home"))),
                    ]
                }
            }
        }
    };

    let layer1_items: Vec<(&str, &str, MdiIcon)> = vec![
        ("Button", "/components/layer1/button", MdiIcon::ToggleSwitch),
        ("Form", "/components/layer1/form", MdiIcon::TextBoxEdit),
        (
            "Number Input",
            "/components/layer1/number-input",
            MdiIcon::FormatListNumbered,
        ),
        ("Search", "/components/layer1/search", MdiIcon::Magnify),
        ("Switch", "/components/layer1/switch", MdiIcon::ToggleSwitch),
        ("Feedback", "/components/layer1/feedback", MdiIcon::Alert),
        ("Display", "/components/layer1/display", MdiIcon::ViewColumn),
        ("Avatar", "/components/layer1/avatar", MdiIcon::Account),
        ("Image", "/components/layer1/image", MdiIcon::Image),
        ("Tag", "/components/layer1/tag", MdiIcon::Tag),
        (
            "Empty",
            "/components/layer1/empty",
            MdiIcon::CheckboxMarkedCircle,
        ),
        ("Comment", "/components/layer1/comment", MdiIcon::Chat),
        (
            "Description List",
            "/components/layer1/description-list",
            MdiIcon::FormatListBulleted,
        ),
    ];

    let layer2_items: Vec<(&str, &str, MdiIcon)> = vec![
        ("Navigation", "/components/layer2/navigation", MdiIcon::Menu),
        (
            "Collapsible",
            "/components/layer2/collapsible",
            MdiIcon::ChevronDown,
        ),
        ("Data", "/components/layer2/data", MdiIcon::Table),
        ("Table", "/components/layer2/table", MdiIcon::Table),
        ("Tree", "/components/layer2/tree", MdiIcon::Tree),
        (
            "Pagination",
            "/components/layer2/pagination",
            MdiIcon::ChevronDoubleRight,
        ),
        (
            "QRCode",
            "/components/layer2/qrcode",
            MdiIcon::ViewDashboard,
        ),
        (
            "Timeline",
            "/components/layer2/timeline",
            MdiIcon::ChartTimeline,
        ),
        ("Form", "/components/layer2/form", MdiIcon::TextBoxEdit),
        (
            "Cascader",
            "/components/layer2/cascader",
            MdiIcon::ChevronDown,
        ),
        (
            "Transfer",
            "/components/layer2/transfer",
            MdiIcon::SwapHorizontal,
        ),
        ("Feedback", "/components/layer2/feedback", MdiIcon::Alert),
    ];

    let layer3_items: Vec<(&str, &str, MdiIcon)> = vec![
        ("Media", "/components/layer3/media", MdiIcon::Play),
        ("Editor", "/components/layer3/editor", MdiIcon::FileEdit),
        (
            "Visualization",
            "/components/layer3/visualization",
            MdiIcon::ChartTimeline,
        ),
        (
            "User Guide",
            "/components/layer3/user-guide",
            MdiIcon::BookOpen,
        ),
        (
            "Zoom Controls",
            "/components/layer3/zoom-controls",
            MdiIcon::MagnifyPlus,
        ),
    ];

    let system_items: Vec<(&str, &str, MdiIcon)> = vec![
        ("Overview", "/system/overview", MdiIcon::Information),
        ("Runtime Engine", "/system/runtime", MdiIcon::CubeOutline),
        ("WIT Pipeline", "/system/wit-pipeline", MdiIcon::FileEdit),
        (
            "Web Backends",
            "/system/web-backends",
            MdiIcon::SourceBranch,
        ),
        ("Versioning", "/system/versioning", MdiIcon::Tag),
        ("CSS Utilities", "/system/css", MdiIcon::FormatAlignLeft),
        ("Icons", "/system/icons", MdiIcon::Star),
        ("Color Palette", "/system/palette", MdiIcon::Palette),
        ("Animations", "/system/animations", MdiIcon::LightningBolt),
        ("i18n", "/system/i18n", MdiIcon::Translate),
        ("Animation Demo", "/animations", MdiIcon::Play),
    ];

    let demo_items: Vec<(&str, &str, MdiIcon)> = vec![
        ("Form Demo", "/demos/form", MdiIcon::TextBoxEdit),
        ("Dashboard", "/demos/dashboard", MdiIcon::ViewDashboard),
        ("Video & Audio", "/demos/video", MdiIcon::Play),
    ];

    let sections: Vec<VNode> = vec![
        home,
        submenu_section("components", MdiIcon::Package, "Components", &[], true),
        submenu_section(
            "layer1",
            MdiIcon::Layers,
            "Layer 1 — Base",
            &layer1_items,
            true,
        ),
        submenu_section(
            "layer2",
            MdiIcon::SourceBranch,
            "Layer 2 — Composed",
            &layer2_items,
            false,
        ),
        submenu_section(
            "layer3",
            MdiIcon::CubeOutline,
            "Layer 3 — Complex",
            &layer3_items,
            false,
        ),
        submenu_section("system", MdiIcon::Cog, "System", &system_items, false),
        submenu_section("demos", MdiIcon::TrophyAward, "Demos", &demo_items, false),
    ];

    VNode::Element(
        el("aside")
            .attr("id", "hikari-aside")
            .class("hi-aside hi-aside-drawer hi-aside-lg")
            .children(sections),
    )
}

fn menu_item(icon: MdiIcon, label: &str, href: &str) -> VNode {
    VNode::Element(
        el("li")
            .attr("role", "menuitem")
            .class("hi-menu-item hi-menu-height-compact")
            .child(VNode::Element(el("a").attr("href", href).children(vec![
                sidebar_icon(icon),
                VNode::Element(el("span").class("hi-sidebar-label").child(txt(label))),
            ]))),
    )
}

// ============================================================
// Breadcrumb Navigation
// ============================================================

pub fn breadcrumb(items: &[(&str, &str)]) -> VNode {
    let mut children: Vec<VNode> = Vec::new();
    for (i, (label, href)) in items.iter().enumerate() {
        if i > 0 {
            children.push(VNode::Element(
                el("span").class("hi-breadcrumb-sep").child(txt(" / ")),
            ));
        }
        if href.is_empty() {
            children.push(VNode::Element(
                el("span").class("hi-breadcrumb-current").child(txt(label)),
            ));
        } else {
            children.push(VNode::Element(
                el("a")
                    .attr("href", href)
                    .class("hi-breadcrumb-link")
                    .child(txt(label)),
            ));
        }
    }
    VNode::Element(el("nav").class("hi-breadcrumb").children(children))
}

// ============================================================
// Aside Footer — SVG icons (moon + lang)
// ============================================================

pub fn aside_footer() -> VNode {
    rsx! {
        div { class: "hi-aside-footer",
            button { class: "hi-button hi-button-borderless hi-icon-button hi-icon-button-40",
                id: "theme-toggle", title: "Toggle theme",
                ..vec![svg_icon(MdiIcon::MoonWaningCrescent, 20, "")]
            }
            button { class: "hi-button hi-button-borderless hi-icon-button hi-icon-button-40",
                id: "lang-toggle", title: "Language", "A"
            }
        }
    }
}
