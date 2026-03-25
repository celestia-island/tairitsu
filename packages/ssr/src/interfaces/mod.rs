//! WIT interface implementations for SSR
//!
//! This module contains implementations of all the WIT interfaces
//! that the SSR host provides to the WASM component.

// Note: console interface removed - console operations now use direct browser console
// via wasm-bindgen in the web package, not WIT interface
// Note: style interface removed - now using W3C CSSOM standard interfaces
// (element-css-inline-style and css-style-declaration) directly in linker.rs

pub mod document;
pub mod element;
pub mod node;
pub mod platform_helpers;
pub mod window;
