//! App root — assembles all pages into a single VNode tree.
//!
//! All pages are rendered at once; JavaScript hash routing controls which
//! page is visible by toggling `.is-active` on `.tairitsu-page` divs.

use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

use crate::{
    components::{sidebar, top_nav},
    pages::{guides, home, not_found, packages, system},
};

/// Render the full application VNode tree.
pub fn render() -> VNode {
    let mut content: Vec<VNode> = Vec::new();
    content.push(home::render());
    content.extend(guides::render_all());
    content.extend(system::render_all());
    content.extend(packages::render_all());
    content.push(not_found::render());

    rsx! {
        div { id: "tairitsu-app", class: "tairitsu-app",
            ..vec![top_nav()],
            div { class: "tairitsu-body",
                ..vec![sidebar()],
                main { class: "tairitsu-content",
                    ..content,
                },
            },
        }
    }
}

pub struct App;

impl App {
    pub fn render(&self) -> VNode {
        render()
    }
}
