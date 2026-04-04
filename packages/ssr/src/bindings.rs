//! WIT bindings for resize-observer-entry using wasmtime bindgen
//!
//! This module uses wasmtime::component::bindgen! to generate type-safe
//! bindings for the resize-observer-entry interface, which properly handles
//! the dom-rect record type.

use wasmtime::component::bindgen;

// Only generate bindings for the specific interface we need
// We use the full browser-full.wit but only use a small part
bindgen!({
    path: "../../packages/browser-worlds/wit/browser-full.wit",
    world: "browser-full",
});

// Re-export the types and traits we need
pub use self::tairitsu_browser::full::component_types::ContentEditableState;
pub use self::tairitsu_browser::full::platform_helpers::Host as PlatformHelpersHost;
pub use self::tairitsu_browser::full::resize_observer_entry::Host as ResizeObserverEntryHost;
pub use self::tairitsu_browser::full::resize_observer_size::Host as ResizeObserverSizeHost;
pub use self::tairitsu_browser::full::types::DomRect;
