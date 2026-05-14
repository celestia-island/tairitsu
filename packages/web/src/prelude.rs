//! Prelude module for tairitsu-web-next
//!
//! This module re-exports all packages based on feature flags

#![allow(ambiguous_glob_reexports)]

// Core runtime (optional - only available with 'runtime' or 'ssr' or 'packager' features)
#[cfg(any(feature = "runtime", feature = "ssr", feature = "packager"))]
pub use tairitsu_core::*;

// Feature-gated package exports
#[cfg(feature = "hooks")]
pub use tairitsu_hooks::*;
#[cfg(feature = "macros")]
pub use tairitsu_macros::*;
#[cfg(feature = "packager")]
pub use tairitsu_packager::*;
#[cfg(feature = "ssr")]
pub use tairitsu_ssr::*;
#[cfg(feature = "style")]
pub use tairitsu_style::*;
#[cfg(feature = "vdom")]
pub use tairitsu_vdom::*;

// Common types and functions
pub use std::cell::RefCell;
pub use std::collections::HashMap;
pub use std::rc::Rc;
