//! Layer 3 — Complex components: Media, Editor, Visualization, etc.

pub mod editor;
pub mod media;
pub mod user_guide;
pub mod visualization;
pub mod zoom_controls;

use tairitsu_vdom::VNode;

pub fn render_all() -> Vec<VNode> {
    vec![
        media::render(),
        editor::render(),
        visualization::render(),
        user_guide::render(),
        zoom_controls::render(),
    ]
}
