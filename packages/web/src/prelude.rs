//! Prelude module for tairitsu-web-next
//!
//! This module re-exports all packages based on feature flags

// Core runtime
pub use tairitsu::*;

// Feature-gated package exports
#[cfg(feature = "vdom")]
pub use tairitsu_vdom::*;
#[cfg(feature = "hooks")]
pub use tairitsu_hooks::*;
#[cfg(feature = "macros")]
pub use tairitsu_macros::*;
#[cfg(feature = "style")]
pub use tairitsu_style::*;
#[cfg(feature = "router")]
pub use tairitsu_router::*;
#[cfg(feature = "data-fetcher")]
pub use tairitsu_data_fetcher::*;
#[cfg(feature = "hmr")]
pub use tairitsu_hmr::*;
#[cfg(feature = "fast-refresh")]
pub use tairitsu_fast_refresh::*;
#[cfg(feature = "ssr")]
pub use tairitsu_ssr::*;
#[cfg(feature = "error-overlay")]
pub use tairitsu_error_overlay::*;
#[cfg(feature = "css-values")]
pub use tairitsu_css_values::*;
#[cfg(feature = "i18n")]
pub use tairitsu_i18n::*;
#[cfg(feature = "packager")]
pub use tairitsu_packager::*;

// Common types and functions
pub use std::rc::Rc;
pub use std::cell::RefCell;
pub use std::collections::HashMap;