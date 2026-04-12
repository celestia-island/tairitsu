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

fn arrow_svg() -> VNode {
    VNode::Element(
        el("span")
            .class("hi-arrow hi-arrow-down hi-arrow-14")
            .child(VNode::Element(
                el("svg")
                    .attr("xmlns", "http://www.w3.org/2000/svg")
                    .attr("viewBox", "0 0 24 24")
                    .attr("fill", "currentColor")
                    .attr("width", "14")
                    .attr("height", "14")
                    .child(VNode::Element(el("path").attr(
                        "d",
                        "M8.59,16.58L13.17,12L8.59,7.41L10,6L16,12L10,18L8.59,16.58Z",
                    ))),
            )),
    )
}

fn submenu_title(label: &str) -> VNode {
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
                                    .child(VNode::Element(
                                        el("span").class("hi-menu-item-content").child(txt(label)),
                                    ))
                                    .child(arrow_svg()),
                            )),
                    )),
            )),
    )
}

fn submenu_list(items: &[(&str, &str)], open: bool) -> VNode {
    let children: Vec<VNode> = items
        .iter()
        .map(|(label, href)| menu_item(label, href))
        .collect();
    let mut list = el("ul")
        .attr("role", "menu")
        .class("hi-menu-submenu-list")
        .attr("style", "padding-left:1em");
    if open {
        list = list.class("hi-menu-submenu-list-open");
        list = list.attr(
            "style",
            "display:block;opacity:1;transform:translateX(0);padding-left:1em",
        );
    }
    VNode::Element(list.children(children))
}

fn submenu_section(key: &str, label: &str, items: &[(&str, &str)], open: bool) -> VNode {
    let mut li = el("li")
        .attr("role", "none")
        .class("hi-menu-submenu")
        .attr("data_key", key);
    if open {
        li = li.class("hi-menu-submenu-list-open");
    }
    let li_el = li
        .child(submenu_title(label))
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
// Sidebar Navigation
// ============================================================

pub fn sidebar() -> VNode {
    let home = rsx! {
        ul { class: "hi-menu hi-menu-vertical hi-menu-compact",
            li { role: "menuitem", class: "hi-menu-item hi-menu-height-compact",
                data_key: "Home",
                div { class: "hi-menu-item-inner",
                    span { class: "hi-menu-item-content", "Home" }
                }
            }
        }
    };

    let guides_items: Vec<(&str, &str)> = vec![
        ("Quick Start", "/guides/quick-start"),
        ("Workspace Map", "/guides/workspace-map"),
        ("Build / Test / Release", "/guides/build-test-release"),
        ("Migration Guide", "/guides/migration"),
        ("Glossary", "/guides/glossary"),
    ];

    let system_items: Vec<(&str, &str)> = vec![
        ("Overview", "/system/overview"),
        ("Runtime Engine", "/system/runtime"),
        ("WIT Pipeline", "/system/wit-pipeline"),
        ("Web Backends", "/system/web-backends"),
        ("Versioning", "/system/versioning"),
    ];

    let packages_items: Vec<(&str, &str)> = vec![
        ("Overview", "/packages/overview"),
        ("Package List", "/packages/list"),
    ];

    let sections: Vec<VNode> = vec![
        home,
        submenu_section("guides", "Guides", &guides_items, true),
        submenu_section("system", "System", &system_items, false),
        submenu_section("packages", "Packages", &packages_items, false),
    ];

    VNode::Element(
        el("aside")
            .attr("id", "hikari-aside")
            .class("hi-aside hi-aside-drawer hi-aside-lg")
            .children(sections),
    )
}

fn menu_item(label: &str, href: &str) -> VNode {
    VNode::Element(
        el("li")
            .attr("role", "menuitem")
            .class("hi-menu-item hi-menu-height-compact")
            .child(VNode::Element(el("a").attr("href", href).child(
                VNode::Element(el("div").class("hi-menu-item-inner").child(VNode::Element(
                    el("span").class("hi-menu-item-content").child(txt(label)),
                ))),
            ))),
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
// Aside Footer
// ============================================================

pub fn aside_footer() -> VNode {
    rsx! {
        div { class: "hi-aside-footer",
            button { class: "hi-button hi-button-borderless hi-icon-button hi-icon-button-40",
                id: "theme-toggle", "\u{2600}"
            }
            button { class: "hi-button hi-button-borderless hi-icon-button hi-icon-button-40",
                id: "lang-toggle", "A"
            }
        }
    }
}
