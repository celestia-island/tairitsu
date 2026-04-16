//! App root — assembles pages into a VNode tree based on current route.
//!
//! Uses hikari's layout CSS classes (hi-layout-*) with tairitsu dark theme.
//! Structure mirrors hikari-legacy website exactly.

use tairitsu_macros::rsx;
use tairitsu_vdom::{VElement, VNode, VText};

use crate::pages::components as page_components;
use crate::{
    components::{aside_footer, sidebar, top_nav},
    pages::{guides, home, not_found, packages, system},
};

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
    rsx! {
        div { id: "hikari-app",
            class: "hi-layout hi-layout-dark hi-layout-has-sidebar",
            div { class: "hi-background" }
            ..vec![top_nav()],
            div { class: "hi-layout-body",
                div { id: "drawer-overlay", class: "hi-layout-overlay" }
                aside { class: "hi-aside hi-aside-drawer hi-aside-lg",
                    div { class: "hi-aside-content", ..vec![sidebar()] }
                    ..vec![aside_footer()]
                }
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
