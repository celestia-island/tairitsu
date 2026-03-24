//! SSR stub implementations for browser WIT interfaces
//!
//! This module provides stub implementations for all browser WIT interfaces
//! that are NOT manually implemented. Stubs return appropriate default values
//! or errors to indicate browser-only operations.
//!
//! Manually implemented interfaces (in linker.rs):
//! - document, node, element, style, console, window, platform-helpers, event-target

use crate::host_state::SsrHostState;
use anyhow::Result;
use wasmtime::component::Linker;

// Include auto-generated stub implementations
// The generated file contains register_all_auto_stubs() function
include!(concat!(env!("OUT_DIR"), "/ssr_stubs_gen.rs"));

/// Register all stub implementations with the linker
///
/// This function registers stub implementations for all browser interfaces
/// that are not manually implemented in linker.rs.
pub fn register_all_stubs(linker: &mut Linker<SsrHostState>) -> Result<()> {
    // Call the auto-generated stub registration function
    register_all_auto_stubs(linker)?;
    Ok(())
}
