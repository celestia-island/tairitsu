//! App root — assembles all pages into a single VNode tree.
//!
//! Uses hikari's layout CSS classes (hi-layout-*) with tairitsu dark theme.
//! Structure mirrors hikari-legacy website exactly.

use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

use crate::{
    components::{aside_footer, sidebar, top_nav},
    pages::{guides, home, not_found, packages, system},
};

pub fn render() -> VNode {
    let mut content: Vec<VNode> = Vec::new();
    content.push(home::render());
    content.extend(guides::render_all());
    content.extend(system::render_all());
    content.extend(packages::render_all());
    content.push(not_found::render());

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
                    main { class: "hi-layout-content", ..content }
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
