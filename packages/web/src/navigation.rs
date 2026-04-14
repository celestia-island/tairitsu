//! Client-side navigation for browser-based Tairitsu apps.
//!
//! Provides URL-aware routing: reads current pathname, pushes history state,
//! intercepts `<a>` click events for SPA-style navigation, and integrates
//! with the reactive system so route changes trigger re-renders.
//!
//! # Example
//!
//! ```ignore
//! use tairitsu_web::navigation;
//! use tairitsu_vdom::VNode;
//!
//! // 1. Start intercepting link clicks inside #app
//! navigation::intercept_links();
//!
//! // 2. Get current path for initial render
//! let path = navigation::current_path();
//!
//! // 3. Later, navigate programmatically
//! navigation::navigate("/components/layer1/button");
//! ```

use crate::wit_platform;

/// Read the current URL pathname.
///
/// Returns `/` if not running in wasm.
pub fn current_path() -> String {
    wit_platform::get_pathname()
}

/// Navigate to a new path (pushState + trigger re-render).
///
/// Updates browser URL via `history.pushState` without page reload.
pub fn navigate(path: &str) {
    wit_platform::push_state(path)
}

/// Replace current URL (replaceState).
pub fn replace(path: &str) {
    wit_platform::replace_state(path)
}
