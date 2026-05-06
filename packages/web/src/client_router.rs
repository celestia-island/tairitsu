//! Client-side router for SPA-style navigation in tairitsu-web apps.
//!
//! This router integrates with the browser's History API for URL-based routing.
//! Click interception is expected to be handled by JS-level code (e.g., in index.html)
//! which calls preventDefault() + pushState(). The Rust-side router handles:
//! - Initial render based on current URL
//! - Re-rendering on URL change (polled every 200ms)
//! - Programmatic navigation via navigate()

use std::cell::RefCell;

use crate::wit_platform::{self, WitPlatform};

thread_local! {
    static INSTANCE: RefCell<Option<RouterServiceInner>> = RefCell::new(None);
    static CURRENT_PATH: RefCell<String> = RefCell::new("/".to_string());
}

struct RouterServiceInner {
    platform: WitPlatform,
    render_fn: Box<dyn Fn(&str) -> tairitsu_vdom::VNode + Send + Sync>,
}

/// Initialize the router: read current URL, render initial route,
/// and start polling for URL changes.
pub fn init_router(
    platform: WitPlatform,
    render_fn: impl Fn(&str) -> tairitsu_vdom::VNode + Send + Sync + 'static,
) {
    let inner = RouterServiceInner {
        platform,
        render_fn: Box::new(render_fn),
    };
    let path = wit_platform::get_pathname();
    do_render(&inner, &path);
    INSTANCE.with(|i| *i.borrow_mut() = Some(inner));
    set_current_path(&path);
    schedule_poll();
}

/// Programmatically navigate to a path (pushState + re-render).
pub fn navigate(path: &str) {
    let normalized = normalize_path(path);
    if normalized == current_path() {
        return;
    }
    wit_platform::push_state(&normalized);
    do_navigate(&normalized);
}

/// Handle a popstate event (back/forward button) with new pathname.
pub fn on_popstate(new_path: &str) {
    let normalized = normalize_path(new_path);
    if normalized == current_path() {
        return;
    }
    do_navigate(&normalized);
}

fn do_navigate(path: &str) {
    INSTANCE.with(|i| {
        if let Some(ref inner) = *i.borrow() {
            do_render(inner, path);
        }
    });
    set_current_path(path);
}

fn do_render(inner: &RouterServiceInner, path: &str) {
    let vnode = (inner.render_fn)(path);
    if let Err(e) = inner.platform.mount_vnode_to_app(vnode) {
        tracing::error!("Failed to mount route '{}': {}", path, e);
    }
}

fn schedule_poll() {
    INSTANCE.with(|i| {
        if let Some(ref inner) = *i.borrow() {
            let _ = tairitsu_vdom::Platform::set_timeout(
                &inner.platform,
                Box::new(|| {
                    poll_url_change();
                }),
                200,
            );
        }
    });
}

fn poll_url_change() {
    let current = wit_platform::get_pathname();
    if current != current_path() {
        let normalized = normalize_path(&current);
        set_current_path(&normalized);
        INSTANCE.with(|i| {
            if let Some(ref inner) = *i.borrow() {
                do_render(inner, &normalized);
            }
        });
    }
    schedule_poll();
}

fn normalize_path(path: &str) -> String {
    let mut p = path.to_string();
    if p.ends_with('/') && p.len() > 1 {
        p.pop();
    }
    if p.is_empty() { "/".to_string() } else { p }
}

pub fn current_path() -> String {
    CURRENT_PATH.with(|p| p.borrow().clone())
}

fn set_current_path(path: &str) {
    CURRENT_PATH.with(|p| *p.borrow_mut() = path.to_string());
}
