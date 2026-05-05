//! Shared layout components: top nav, sidebar, aside footer, and glow wrapper.

use hikari_icons::{get, MdiIcon};
use std::cell::Cell;
use tairitsu_macros::rsx;
use tairitsu_vdom::{get_bounding_client_rect, set_style, svg::SafeSvg, DomHandle, VElement, VNode, el, txt};

use crate::i18n::{self, Language};

thread_local! {
    static LANG_OPEN: Cell<bool> = const { Cell::new(false) };
}


/// Render an MDI SVG icon as a VNode using VElement builder.
pub fn svg_icon(icon: MdiIcon, size: u32, class: &str) -> VNode {
    let name = icon.to_string();
    let (view_box, path_d) = match get(&name) {
        Some(data) => (
            data.view_box.as_deref().unwrap_or("0 0 24 24"),
            data.path.as_deref().unwrap_or(""),
        ),
        None => ("0 0 24 24", "M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-2h2v2zm0-4h-2V7h2v6z"),
    };
    let full_class = if class.is_empty() {
        "ts-icon".to_string()
    } else {
        format!("ts-icon {}", class)
    };
    let svg_html = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="{}"><path fill="currentColor" d="{}"/></svg>"#,
        view_box, path_d
    );
    VNode::Element(
        el("div")
            .class(full_class.as_str())
            .attr("style", &format!("width:{}px;height:{}px;", size, size))
            .safe_svg(SafeSvg::from_static(Box::leak(svg_html.into_boxed_str()))),
    )
}

fn icon_el(icon: MdiIcon) -> VNode {
    let icon_name = icon.to_string();
    let svg_str = get(&icon_name)
        .map(|data| {
            format!(
                r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="{}" width="14" height="14"><path fill="currentColor" d="{}"/></svg>"#,
                data.view_box.as_deref().unwrap_or("0 0 24 24"),
                data.path.as_deref().unwrap_or("")
            )
        })
        .unwrap_or_else(|| String::from(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24"><path fill="currentColor" d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-2h2v2zm0-4h-2V7h2v6z"/></svg>"#
        ));
    VNode::Element(
        el("span")
            .class("hi-menu-item-icon ts-icon")
            .inner_html(svg_str),
    )
}

/// Glow effect wrapper with native Rust mouse-tracking.
pub fn glow_wrapper(blur: &str, intensity: &str, color: &str, children: VNode) -> VNode {
    let opacity = match intensity {
        "dim" => "0.07",
        "soft" => "0.15",
        "bright" => "0.30",
        "subtle" => "0.10",
        _ => "0.15",
    };

    let onmousemove = move |e: std::boxed::Box<dyn tairitsu_vdom::EventData>| {
        if let Some(me) = e.as_any().downcast_ref::<tairitsu_vdom::MouseEvent>() {
            if let Some(target) = me.target {
                let handle = DomHandle::from_raw(target);
                let rect = get_bounding_client_rect(handle);
                if rect.width > 0.0 && rect.height > 0.0 {
                    let px = (me.offset_x as f64 / rect.width * 100.0).clamp(0.0, 100.0);
                    let py = (me.offset_y as f64 / rect.height * 100.0).clamp(0.0, 100.0);
                    set_style(handle, "--glow-x", &format!("{:.1}%", px));
                    set_style(handle, "--glow-y", &format!("{:.1}%", py));
                }
            }
        }
    };
    let onmouseenter = move |e: std::boxed::Box<dyn tairitsu_vdom::EventData>| {
        if let Some(me) = e.as_any().downcast_ref::<tairitsu_vdom::MouseEvent>() {
            if let Some(target) = me.target {
                set_style(DomHandle::from_raw(target), "--glow-opacity", opacity);
            }
        }
    };
    let onmouseleave = move |e: std::boxed::Box<dyn tairitsu_vdom::EventData>| {
        if let Some(me) = e.as_any().downcast_ref::<tairitsu_vdom::MouseEvent>() {
            if let Some(target) = me.target {
                let handle = DomHandle::from_raw(target);
                set_style(handle, "--glow-x", "50%");
                set_style(handle, "--glow-y", "50%");
                set_style(handle, "--glow-opacity", "0");
            }
        }
    };

    VNode::Element(
        el("div")
            .class(format!("hi-glow-wrapper hi-glow-blur-{} hi-glow-{}", blur, intensity).as_str())
            .attr("style", &format!(
                "--glow-x:50%;--glow-y:50%;--glow-color:{};--glow-opacity:0;--glow-intensity-scale:0;",
                color
            ))
            .on_event("mousemove", onmousemove)
            .on_event("mouseenter", onmouseenter)
            .on_event("mouseleave", onmouseleave)
            .child(children),
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
                    "aria-label": "Toggle menu",
                    svg { xmlns: "http://www.w3.org/2000/svg", fill: "none", viewBox: "0 0 24 24",
                        stroke: "currentColor", stroke_width: "2", stroke_linecap: "round", stroke_linejoin: "round",
                        path { d: "M4 6h16M4 12h16M4 18h16" }
                    }
                }
                a { href: "/", class: "hi-header-brand",
                    img {
                        class: "hi-header-logo-img",
                        src: "/images/logo-tairitsu.png",
                        alt: "Tairitsu",
                        width: "28",
                        height: "28"
                    }
                    span { style: "font-weight:700;font-size:1.15rem;margin-left:8px;", "Tairitsu" }
                }
            }
            div { class: "hi-header-right",
                nav { class: "hi-header-nav",
                    a { href: "/guides/quick-start", class: "ts-topnav__link", "Guides" }
                    a { href: "/system", class: "ts-topnav__link", "System" }
                    a { href: "/packages", class: "ts-topnav__link", "Packages" }
                }
                a {
                    href: "https://github.com/langyo/tairitsu",
                    target: "_blank",
                    class: "hi-header-github",
                    "GitHub"
                }
            }
        }
    }
}

// ============================================================
// Sidebar Navigation — mirrors hikari sidebar structure
// ============================================================

struct NavItem {
    label_key: &'static str,
    icon: MdiIcon,
    href: &'static str,
}

struct NavSubcategory {
    label_key: &'static str,
    href: &'static str,
    items: &'static [NavItem],
}

struct NavCategory {
    label_key: &'static str,
    default_open: bool,
    subcategories: &'static [NavSubcategory],
}

fn sidebar_label<'a>(key: &'a str, t: &'a i18n::SiteText) -> &'a str {
    match key {
        "sidebar_home" => t.sidebar_home,
        "sidebar_getting_started" => t.sidebar_getting_started,
        "sidebar_architecture" => t.sidebar_architecture,
        "sidebar_packages_category" => t.sidebar_packages_category,
        "sidebar_system" => t.sidebar_system,
        "sidebar_quick_start" => t.sidebar_quick_start,
        "sidebar_debug_api" => t.sidebar_debug_api,
        "sidebar_workspace_map" => t.sidebar_workspace_map,
        "sidebar_build_test" => t.sidebar_build_test,
        "sidebar_migration" => t.sidebar_migration,
        "sidebar_glossary" => t.sidebar_glossary,
        "sidebar_core_packages" => t.sidebar_core_packages,
        "sidebar_runtime_engine" => t.sidebar_runtime_engine,
        "sidebar_vnode_vdom" => t.sidebar_vnode_vdom,
        "sidebar_reactive_hooks" => t.sidebar_reactive_hooks,
        "sidebar_web_platform" => t.sidebar_web_platform,
        "sidebar_web_backends" => t.sidebar_web_backends,
        "sidebar_browser_worlds" => t.sidebar_browser_worlds,
        "sidebar_wit_pipeline" => t.sidebar_wit_pipeline,
        "sidebar_tooling" => t.sidebar_tooling,
        "sidebar_packager" => t.sidebar_packager,
        "sidebar_style_system" => t.sidebar_style_system,
        "sidebar_browser_glue" => t.sidebar_browser_glue,
        "sidebar_e2e_tests" => t.sidebar_e2e_tests,
        "sidebar_overview" => t.sidebar_overview,
        "sidebar_wit_pipeline_item" => t.sidebar_wit_pipeline_item,
        "sidebar_web_backends_item" => t.sidebar_web_backends_item,
        "sidebar_versioning" => t.sidebar_versioning,
        "sidebar_all_packages" => t.sidebar_all_packages,
        _ => key,
    }
}

const NAV_CATEGORIES: &[NavCategory] = &[
    NavCategory {
        label_key: "sidebar_home",
        default_open: true,
        subcategories: &[NavSubcategory {
            label_key: "sidebar_home",
            href: "/",
            items: &[],
        }],
    },
    NavCategory {
        label_key: "sidebar_getting_started",
        default_open: true,
        subcategories: &[
            NavSubcategory { label_key: "sidebar_quick_start", href: "/guides/quick-start", items: &[] },
            NavSubcategory { label_key: "sidebar_debug_api", href: "/guides/debug-api", items: &[] },
            NavSubcategory { label_key: "sidebar_workspace_map", href: "/guides/workspace-map", items: &[] },
            NavSubcategory { label_key: "sidebar_build_test", href: "/guides/build-test-release", items: &[] },
            NavSubcategory { label_key: "sidebar_migration", href: "/guides/migration", items: &[] },
            NavSubcategory { label_key: "sidebar_glossary", href: "/guides/glossary", items: &[] },
        ],
    },
    NavCategory {
        label_key: "sidebar_architecture",
        default_open: true,
        subcategories: &[
            NavSubcategory {
                label_key: "sidebar_core_packages",
                href: "/system/runtime",
                items: &[
                    NavItem { label_key: "sidebar_runtime_engine", icon: MdiIcon::Cog, href: "/system/runtime" },
                    NavItem { label_key: "sidebar_vnode_vdom", icon: MdiIcon::FileEdit, href: "/packages/list" },
                    NavItem { label_key: "sidebar_reactive_hooks", icon: MdiIcon::Code, href: "/packages" },
                ],
            },
            NavSubcategory {
                label_key: "sidebar_web_platform",
                href: "/system/web-backends",
                items: &[
                    NavItem { label_key: "sidebar_web_backends", icon: MdiIcon::CubeOutline, href: "/system/web-backends" },
                    NavItem { label_key: "sidebar_browser_worlds", icon: MdiIcon::Layers, href: "/system" },
                    NavItem { label_key: "sidebar_wit_pipeline", icon: MdiIcon::SourceBranch, href: "/system/wit-pipeline" },
                ],
            },
            NavSubcategory {
                label_key: "sidebar_tooling",
                href: "/packages/list",
                items: &[
                    NavItem { label_key: "sidebar_packager", icon: MdiIcon::Package, href: "/packages/list" },
                    NavItem { label_key: "sidebar_style_system", icon: MdiIcon::Palette, href: "/packages" },
                    NavItem { label_key: "sidebar_browser_glue", icon: MdiIcon::Code, href: "/system/runtime" },
                    NavItem { label_key: "sidebar_e2e_tests", icon: MdiIcon::CheckboxMarkedCircle, href: "/system/versioning" },
                ],
            },
        ],
    },
    NavCategory {
        label_key: "sidebar_packages_category",
        default_open: false,
        subcategories: &[NavSubcategory { label_key: "sidebar_all_packages", href: "/packages/list", items: &[] }],
    },
    NavCategory {
        label_key: "sidebar_system",
        default_open: false,
        subcategories: &[
            NavSubcategory { label_key: "sidebar_overview", href: "/system", items: &[] },
            NavSubcategory { label_key: "sidebar_runtime_engine", href: "/system/runtime", items: &[] },
            NavSubcategory { label_key: "sidebar_wit_pipeline_item", href: "/system/wit-pipeline", items: &[] },
            NavSubcategory { label_key: "sidebar_web_backends_item", href: "/system/web-backends", items: &[] },
            NavSubcategory { label_key: "sidebar_versioning", href: "/system/versioning", items: &[] },
        ],
    },
];

pub fn sidebar() -> VNode {
    let t = i18n::text(Language::default_lang());
    let mut category_nodes: Vec<VNode> = Vec::new();

    for category in NAV_CATEGORIES {
        if category.subcategories.len() == 1 && category.subcategories[0].items.is_empty() {
            let sub = &category.subcategories[0];
            category_nodes.push(plain_menu_item(sub.href, sidebar_label(sub.label_key, &t), 1));
            continue;
        }

        let mut subcategory_nodes: Vec<VNode> = Vec::new();

        for subcategory in category.subcategories {
            if subcategory.items.is_empty() {
                subcategory_nodes.push(plain_menu_item(subcategory.href, sidebar_label(subcategory.label_key, &t), 2));
            } else {
                let item_nodes: Vec<VNode> = subcategory
                    .items
                    .iter()
                    .map(|item| menu_item(item.href, sidebar_label(item.label_key, &t), item.icon))
                    .collect();
                subcategory_nodes.push(submenu(sidebar_label(subcategory.label_key, &t), 2, item_nodes, true));
            }
        }

        category_nodes.push(submenu(
            sidebar_label(category.label_key, &t),
            1,
            subcategory_nodes,
            category.default_open,
        ));
    }

    let menu_list = VNode::Element(
        el("ul")
            .class("hi-menu-list")
            .children(category_nodes),
    );

    VNode::Element(
        el("aside")
            .attr("id", "ts-aside")
            .class("hi-aside hi-aside-drawer hi-aside-lg hi-aside-light")
            .child(VNode::Element(
                el("div")
                    .class("hi-aside-content")
                    .child(VNode::Element(
                        el("nav")
                            .class("hi-menu hi-menu-vertical hi-menu-compact")
                            .child(menu_list),
                    )),
            ))
            .child(aside_footer()),
    )
}

fn menu_item(href: &str, label: &str, icon: MdiIcon) -> VNode {
    let inner = VNode::Element(
        el("a")
            .attr("href", href)
            .class("hi-menu-item-inner")
            .child(icon_el(icon))
            .child(VNode::Element(
                el("span")
                    .class("hi-menu-item-content")
                    .child(txt(label)),
            )),
    );
    glow_wrapper(
        "medium",
        "dim",
        "rgba(128,128,128,0.3)",
        VNode::Element(
            el("li")
                .class("hi-menu-item hi-menu-height-compact")
                .child(inner),
        ),
    )
}

fn submenu_title(label: &str, _level: u32) -> VNode {
    let arrow = icon_el(MdiIcon::ChevronRight);
    VNode::Element(
        el("div")
            .class(format!("hi-submenu-title hi-menu-height-compact"))
            .child(arrow)
            .child(txt(label)),
    )
}

fn submenu(label: &str, level: u32, children: Vec<VNode>, open: bool) -> VNode {
    let title = submenu_title(label, level);
    let list = VNode::Element(
        el("ul")
            .class("hi-submenu-list")
            .style(format!("padding-left:{}em", level))
            .children(children),
    );
    let mut el = VElement::new("li").class("hi-submenu");
    el = el.child(title).child(list);
    if !open {
        el = el.attr("data-collapsed", "");
    }
    VNode::Element(el)
}

fn plain_menu_item(href: &str, label: &str, _level: u32) -> VNode {
    let inner = VNode::Element(
        el("a")
            .attr("href", href)
            .class("hi-menu-item-inner")
            .child(VNode::Element(
                el("span")
                    .class("hi-menu-item-content")
                    .child(txt(label)),
            )),
    );
    glow_wrapper(
        "medium",
        "dim",
        "rgba(128,128,128,0.3)",
        VNode::Element(
            el("li")
                .class(format!("hi-menu-item hi-menu-height-compact"))
                .child(inner),
        ),
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
    let t = i18n::text(Language::default_lang());
    let lang_open = LANG_OPEN.with(|c| c.get());
    rsx! {
        div { class: "hi-aside-footer",
            button { class: "hi-aside-footer__btn", id: "dark-mode-toggle", title: t.toggle_theme,
                onclick: move |_e: tairitsu_vdom::MouseEvent| {
                    crate::app::toggle_dark_mode();
                },
                span { class: "hi-aside-footer__icon", "\u{263E}" }
            }
            div { id: "lang-selector",
                class: "hi-select hi-select-sm",
                div {
                    class: "hi-select-trigger hi-select-sm",
                    onclick: move |_e: tairitsu_vdom::MouseEvent| {
                        LANG_OPEN.with(|c| c.set(!c.get()));
                        tairitsu_vdom::rerender();
                    },
                    span { class: "hi-select-value", "A" }
                    span { class: "hi-select-arrow",
                        inner_html: r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M6 9l6 6 6-6"/></svg>"#
                    }
                }
                ..if lang_open {
                    vec![VNode::Element(
                        el("div")
                            .class("hi-select-dropdown")
                            .children(vec![
                                VNode::Element(el("div").class("hi-select-option").child(txt("English"))),
                                VNode::Element(el("div").class("hi-select-option").child(txt("简体中文"))),
                                VNode::Element(el("div").class("hi-select-option").child(txt("繁體中文"))),
                                VNode::Element(el("div").class("hi-select-option").child(txt("日本語"))),
                                VNode::Element(el("div").class("hi-select-option").child(txt("한국어"))),
                            ])
                    )]
                } else {
                    vec![]
                }
            }
        }
    }
}
