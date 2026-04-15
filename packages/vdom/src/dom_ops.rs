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

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

use crate::platform::DomRect;

static DOM_OPS_CALLBACK_ID: AtomicU64 = AtomicU64::new(1);

thread_local! {
    static RAF_CALLBACKS: RefCell<HashMap<u64, Box<dyn FnOnce(f64)>>> = RefCell::new(HashMap::new());
}

// Global function pointers for WIT operations
static WIT_FUNCS: Mutex<Option<WitFuncs>> = Mutex::new(None);

struct WitFuncs {
    set_style: unsafe fn(u64, &str, &str) -> Result<(), String>,
    get_bounding_client_rect: unsafe fn(u64) -> DomRect,
    set_attribute: unsafe fn(u64, &str, &str),
    request_animation_frame: unsafe fn(u64) -> u32,
    dispatch_animation_frame: unsafe fn(u64, f64),
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
    request_animation_frame: unsafe fn(u64) -> u32,
    dispatch_animation_frame: unsafe fn(u64, f64),
) {
    *WIT_FUNCS.lock().unwrap() = Some(WitFuncs {
        set_style,
        get_bounding_client_rect,
        set_attribute,
        request_animation_frame,
        dispatch_animation_frame,
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

/// Request an animation frame callback.
///
/// The callback is invoked once with the current timestamp (ms since page load).
/// For a continuous loop, re-register from within the callback.
///
/// # Example
///
/// ```ignore
/// fn start_loop() {
///     request_animation_frame(Box::new(move |_ts| {
///         // do work...
///         start_loop(); // re-register for next frame
///     }));
/// }
/// ```
pub fn request_animation_frame(callback: Box<dyn FnOnce(f64)>) -> u32 {
    if let Some(funcs) = WIT_FUNCS.lock().unwrap().as_ref() {
        let id = DOM_OPS_CALLBACK_ID.fetch_add(1, Ordering::SeqCst);
        RAF_CALLBACKS.with(|m| {
            m.borrow_mut().insert(id, callback);
        });
        unsafe { (funcs.request_animation_frame)(id) }
    } else {
        0
    }
}

/// Dispatch a dom_ops animation frame callback.
///
/// Called from the WIT export `on_frame` to route callbacks that were
/// registered via [`request_animation_frame`].
pub fn dispatch_dom_ops_animation_frame(callback_id: u64, timestamp: f64) {
    RAF_CALLBACKS.with(|m| {
        if let Some(callback) = m.borrow_mut().remove(&callback_id) {
            callback(timestamp);
        }
    });
}
