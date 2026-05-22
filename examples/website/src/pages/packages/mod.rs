//! Package listing pages for tairitsu framework.

use tairitsu_vdom::{txt, VElement, VNode};

use crate::components::breadcrumb;

pub fn render_all() -> Vec<VNode> {
    vec![render_overview(), render_list()]
}

pub fn render_overview() -> VNode {
    let cards: Vec<VNode> = vec![
        package_card_vnode(
            "tairitsu-vdom",
            "Virtual DOM",
            "Cross-platform VNode system with builder API and rsx! macro support",
        ),
        package_card_vnode(
            "tairitsu-hooks",
            "Reactive Hooks",
            "Signal-based reactivity system (use_signal, use_effect, use_memo)",
        ),
        package_card_vnode(
            "tairitsu-macros",
            "Procedural Macros",
            "rsx! JSX-like macro, define_props, and code generation",
        ),
        package_card_vnode(
            "tairitsu-web",
            "Web Platform",
            "WASI browser bindings, WitPlatform, and DOM integration",
        ),
        package_card_vnode(
            "tairitsu-style",
            "Style System",
            "CSS-in-Rust styling with typed classes and property builder",
        ),
        package_card_vnode(
            "tairitsu-packager",
            "CLI Packager",
            "Build toolchain: dev server, WASM packaging, asset pipeline",
        ),
        package_card_vnode(
            "browser-glue",
            "Browser Glue",
            "TypeScript glue layer generated from WIT interfaces",
        ),
        package_card_vnode(
            "browser-worlds",
            "Browser Worlds",
            "WIT world definitions for Web APIs (DOM, fetch, etc.)",
        ),
    ];

    VNode::Element(
        VElement::new("div")
            .attr("id", "page-packages-overview")
            .class("hikari-page")
            .children(vec![
                breadcrumb(&[("Home", "/"), ("Packages", "")]),
                VNode::Element(
                    VElement::new("section").class("page-hero").children(vec![
                        VNode::Element(VElement::new("h1").child(txt("Packages"))),
                        VNode::Element(VElement::new("p").child(txt("Tairitsu is organized into specialized crates, each addressing a specific concern in the WASM Component ecosystem."))),
                    ]),
                ),
                VNode::Element(
                    VElement::new("div")
                        .class("package-grid")
                        .children(cards),
                ),
            ]),
    )
}

pub fn render_list() -> VNode {
    let packages = [
        (
            "tairitsu-vdom",
            "Virtual DOM — VNode, VElement, VText, builder API, event types",
        ),
        (
            "tairitsu-hooks",
            "Reactive Hooks — use_signal, use_effect, use_memo, use_context",
        ),
        (
            "tairitsu-macros",
            "Procedural Macros — rsx!, define_props, component macro",
        ),
        (
            "tairitsu-web",
            "Web Platform — WitPlatform, mount_vnode_to_app, browser WIT bindings",
        ),
        (
            "tairitsu-style",
            "Style System — Style struct, ClassesBuilder, CSS properties",
        ),
        (
            "tairitsu-packager",
            "CLI Packager — dev server, build, asset pipeline, watch mode",
        ),
        (
            "browser-glue",
            "Browser Glue — TypeScript runtime glue from WIT interfaces",
        ),
        (
            "browser-worlds",
            "Browser Worlds — WIT world definitions (dom, window, console)",
        ),
        (
            "browser-wit-resolver",
            "WIT Resolver — Package resolution and caching for WIT files",
        ),
        (
            "browser-test",
            "Browser Test — Chromium automation for E2E screenshot testing",
        ),
        (
            "runtime",
            "Runtime Core — Component instantiation, lifecycle, host integration",
        ),
    ];

    let mut items: Vec<VNode> = Vec::new();
    for (name, desc) in packages {
        items.push(VNode::Element(
            VElement::new("div")
                .class("package-list-item")
                .children(vec![
                    VNode::Element(VElement::new("code").class("package-name").child(txt(name))),
                    VNode::Element(VElement::new("span").class("package-desc").child(txt(desc))),
                ]),
        ));
    }

    VNode::Element(
        VElement::new("div")
            .attr("id", "page-packages-list")
            .class("hikari-page")
            .children(vec![
                breadcrumb(&[
                    ("Home", "/"),
                    ("Packages", "/packages"),
                    ("Package List", ""),
                ]),
                VNode::Element(VElement::new("section").class("page-hero").children(vec![
                        VNode::Element(VElement::new("h1").child(txt("Package List"))),
                        VNode::Element(
                            VElement::new("p")
                                .child(txt("Complete listing of all tairitsu workspace crates.")),
                        ),
                    ])),
                VNode::Element(VElement::new("div").class("doc-content").children(items)),
            ]),
    )
}

fn package_card_vnode(name: &str, title: &str, desc: &str) -> VNode {
    VNode::Element(VElement::new("div").class("package-card").children(vec![
                VNode::Element(
                    VElement::new("div")
                        .class("package-card-header")
                        .children(vec![
                            VNode::Element(VElement::new("code").child(txt(name))),
                            VNode::Element(VElement::new("h3").child(txt(title))),
                        ]),
                ),
                VNode::Element(
                    VElement::new("p")
                        .class("package-card-desc")
                        .child(txt(desc)),
                ),
                VNode::Element(
                    VElement::new("a")
                        .attr("href", "#")
                        .class("hi-button hi-button-ghost hi-button-sm")
                        .child(txt("View Docs →")),
                ),
            ]))
}
