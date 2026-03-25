//! Global DOM operations for use in event handlers.
//!
//! When an event handler receives an event with a target element handle,
//! these functions can be used to directly manipulate the DOM without
//! requiring a Platform reference.
//!
//! # Initialization
//!
//! Before using these functions in a WIT environment, you must call
//! [`register_wit_functions`] to provide the WIT binding functions.

use crate::platform::DomRect;
use std::sync::Mutex;

// Global function pointers for WIT operations
static WIT_FUNCS: Mutex<Option<WitFuncs>> = Mutex::new(None);

struct WitFuncs {
    set_style: unsafe fn(u64, &str, &str) -> Result<(), String>,
    get_bounding_client_rect: unsafe fn(u64) -> DomRect,
    set_attribute: unsafe fn(u64, &str, &str),
}

/// Register the WIT binding functions for DOM operations.
///
/// This should be called once during initialization, typically in
/// the component's bootstrap function.
///
/// # Safety
///
/// The caller must ensure that the provided function pointers are valid
/// and will remain valid for the lifetime of the program.
pub unsafe fn register_wit_functions(
    set_style: unsafe fn(u64, &str, &str) -> Result<(), String>,
    get_bounding_client_rect: unsafe fn(u64) -> DomRect,
    set_attribute: unsafe fn(u64, &str, &str),
) {
    *WIT_FUNCS.lock().unwrap() = Some(WitFuncs {
        set_style,
        get_bounding_client_rect,
        set_attribute,
    });
}

/// Set a CSS property on an element by handle.
///
/// This is a convenience function for event handlers that have access
/// to the target element handle but not a Platform reference.
///
/// # Example
///
/// ```ignore
/// onmouseenter: move |e: MouseEvent| {
///     if let Some(target) = e.target {
///         set_style(target, "--glow-x", "100");
///         set_style(target, "--glow-y", "200");
///     }
/// }
/// ```
pub fn set_style(element_handle: u64, property: &str, value: &str) {
    if let Some(funcs) = WIT_FUNCS.lock().unwrap().as_ref() {
        unsafe {
            let _ = (funcs.set_style)(element_handle, property, value);
        }
    }
}

/// Get the bounding client rect of an element by handle.
///
/// Returns the element's size and position relative to the viewport.
///
/// # Example
///
/// ```ignore
/// onmouseenter: move |e: MouseEvent| {
///     if let Some(target) = e.target {
///         let rect = get_bounding_client_rect(target);
///         println!("Element position: {}, {}", rect.x, rect.y);
///     }
/// }
/// ```
pub fn get_bounding_client_rect(element_handle: u64) -> DomRect {
    if let Some(funcs) = WIT_FUNCS.lock().unwrap().as_ref() {
        unsafe { (funcs.get_bounding_client_rect)(element_handle) }
    } else {
        DomRect {
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
        }
    }
}

/// Set an attribute on an element by handle.
pub fn set_attribute(element_handle: u64, name: &str, value: &str) {
    if let Some(funcs) = WIT_FUNCS.lock().unwrap().as_ref() {
        unsafe { (funcs.set_attribute)(element_handle, name, value) };
    }
}
