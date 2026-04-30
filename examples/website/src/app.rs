//! App root — assembles pages into a VNode tree based on current route.

use std::cell::Cell;

use tairitsu_macros::rsx;
use tairitsu_vdom::{VElement, VNode, VText};

thread_local! {
    static DARK_MODE: Cell<bool> = const { Cell::new(true) };
}

pub fn is_dark_mode() -> bool {
    DARK_MODE.with(|c| c.get())
}

pub fn toggle_dark_mode() {
    DARK_MODE.with(|c| c.set(!c.get()));
    tairitsu_vdom::rerender();
}
use crate::{
    components::{sidebar, top_nav},
    pages::{event_test, guides, home, not_found, packages, system},
    theme,
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
    content.extend(guides::render_all());
    content.extend(system::render_all());
    content.push(event_test::render());
    content.extend(packages::render_all());
    content.push(not_found::render());
    layout_shell(content)
}

fn layout_shell(children: Vec<VNode>) -> VNode {
    let theme_style = theme::tairitsu_style();
    let dark_class = if is_dark_mode() { " hi-layout-dark" } else { "" };
    let layout_class = format!("hi-layout{} hi-layout-has-sidebar hi-ambient-bg", dark_class);
    rsx! {
        div { id: "hikari-app",
            class: layout_class,
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
