//! App root — assembles pages into a VNode tree based on current route.
//!
//! Uses hikari's layout CSS classes (hi-layout-*) with tairitsu dark theme
//! injected via hikari's palette system (theme::tairitsu_style).
//! Structure mirrors hikari website exactly.

use tairitsu_macros::rsx;
use tairitsu_vdom::{VElement, VNode, VText};

use crate::pages::components as page_components;
use crate::{
    components::{sidebar, top_nav},
    pages::{guides, home, not_found, packages, system},
    theme,
};

pub fn rerender() {
    #[cfg(target_family = "wasm")]
    {
        if let Ok(platform) = tairitsu_web::WitPlatform::new() {
            let _ = platform.mount_vnode_to_app(&App.render());
        }
    }
}

fn txt(s: &str) -> VNode {
    VNode::Text(VText::new(s))
}
fn el(tag: &str) -> VElement {
    VElement::new(tag)
}

/// Render the full app — all pages included for JS-based SPA show/hide.
pub fn render() -> VNode {
    let mut content: Vec<VNode> = Vec::new();
    content.push(home::render());
    content.extend(page_components::render_all());
    content.extend(guides::render_all());
    content.extend(system::render_all());
    content.extend(packages::render_all());
    content.push(not_found::render());
    layout_shell(content)
}

fn layout_shell(children: Vec<VNode>) -> VNode {
    let theme_style = theme::tairitsu_style();
    rsx! {
        div { id: "hikari-app",
            class: "hi-layout hi-layout-dark hi-layout-has-sidebar hi-ambient-bg",
            style: theme_style,
            div { class: "hi-background" }
            ..vec![top_nav()],
                div { class: "hi-layout-body",
                    div { id: "drawer-overlay", class: "hi-layout-overlay" }
                    ..vec![sidebar()]
                div { class: "hi-layout-main",
                    main { class: "hi-layout-content",
                        ..children
                    }
                }
            }
        }
    }
}

pub struct App;

impl App {
    pub fn render(&self) -> VNode {
        render()
    }
}
