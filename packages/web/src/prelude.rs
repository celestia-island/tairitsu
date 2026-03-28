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
#[cfg(feature = "ssr")]
pub use tairitsu_ssr::*;
#[cfg(feature = "packager")]
pub use tairitsu_packager::*;

// Common types and functions
pub use std::rc::Rc;
pub use std::cell::RefCell;
pub use std::collections::HashMap;