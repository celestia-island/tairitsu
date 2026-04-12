//! Shared layout components: top nav, sidebar, and aside footer.
//!
//! Mirrors hikari-legacy's component structure exactly,
//! but renders under tairitsu dark theme.

use tairitsu_macros::rsx;
use tairitsu_vdom::{VElement, VNode, VText};

fn txt(s: &str) -> VNode {
    VNode::Text(VText::new(s))
}
fn el(tag: &str) -> VElement {
    VElement::new(tag)
}

fn sidebar_icon(glyph: &str) -> VNode {
    VNode::Element(el("span").class("hi-sidebar-icon").child(txt(glyph)))
}

fn arrow_indicator() -> VNode {
    VNode::Element(el("span").class("hi-sidebar-arrow").child(txt("\u{25B6}")))
}

fn submenu_title(icon: &str, label: &str) -> VNode {
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
                            .child(VNode::Element(
                                el("div")
                                    .class("hi-menu-submenu-title-inner")
                                    .child(sidebar_icon(icon))
                                    .child(arrow_indicator())
                                    .child(VNode::Element(
                                        el("span").class("hi-menu-item-content").child(txt(label)),
                                    )),
                            )),
                    )),
            )),
    )
}

fn submenu_list(items: &[(&str, &str, &str)], open: bool) -> VNode {
    let children: Vec<VNode> = items
        .iter()
        .map(|(icon, label, href)| menu_item(icon, label, href))
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
    icon: &str,
    label: &str,
    items: &[(&str, &str, &str)],
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
// Sidebar Navigation — mirrors hikari-legacy exactly
// Unicode icons match hikari's sidebar-icon system
// ============================================================

pub fn sidebar() -> VNode {
    let home = rsx! {
        ul { class: "hi-menu hi-menu-vertical hi-menu-compact",
            li { role: "menuitem", class: "hi-menu-item hi-menu-item--home hi-menu-height-compact",
                data_key: "Home",
                a { href: "/",
                    span { class: "hi-sidebar-icon", "\u{2302}" }
                    span { class: "hi-sidebar-label", "Home" }
                }
            }
        }
    };

    let guides_items: Vec<(&str, &str, &str)> = vec![
        ("\u{25C9}", "Quick Start", "/guides/quick-start"),
        ("\u{25A6}", "Workspace Map", "/guides/workspace-map"),
        (
            "\u{25A2}",
            "Build / Test / Release",
            "/guides/build-test-release",
        ),
        ("\u{2194}", "Migration Guide", "/guides/migration"),
        ("\u{270E}", "Glossary", "/guides/glossary"),
    ];

    let system_items: Vec<(&str, &str, &str)> = vec![
        ("\u{25C9}", "Overview", "/system/overview"),
        ("\u{2699}", "Runtime Engine", "/system/runtime"),
        ("\u{25C8}", "WIT Pipeline", "/system/wit-pipeline"),
        ("\u{25A6}", "Web Backends", "/system/web-backends"),
        ("\u{25CB}", "Versioning", "/system/versioning"),
    ];

    let packages_items: Vec<(&str, &str, &str)> = vec![
        ("\u{229E}", "Overview", "/packages/overview"),
        ("\u{2261}", "Package List", "/packages/list"),
    ];

    let sections: Vec<VNode> = vec![
        home,
        submenu_section("guides", "\u{229E}", "Guides", &guides_items, true),
        submenu_section("system", "\u{2699}", "System", &system_items, false),
        submenu_section("packages", "\u{229E}", "Packages", &packages_items, false),
    ];

    VNode::Element(
        el("aside")
            .attr("id", "hikari-aside")
            .class("hi-aside hi-aside-drawer hi-aside-lg")
            .children(sections),
    )
}

fn menu_item(icon: &str, label: &str, href: &str) -> VNode {
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
// Aside Footer — matches hikari-legacy footer style
// Uses ☾ moon icon (like hikari light mode → switch to dark)
// ============================================================

pub fn aside_footer() -> VNode {
    rsx! {
        div { class: "hi-aside-footer",
            button { class: "hi-button hi-button-borderless hi-icon-button hi-icon-button-40",
                id: "theme-toggle", title: "Toggle theme", "\u{263E}"
            }
            button { class: "hi-button hi-button-borderless hi-icon-button hi-icon-button-40",
                id: "lang-toggle", title: "Language", "A"
            }
        }
    }
}
