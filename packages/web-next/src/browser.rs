//! Browser-specific functionality for Tairitsu
//!
//! This module contains the browser-specific implementations that were previously
//! in the browser-glue TypeScript package.

use wasm_bindgen::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

/// Browser-specific helpers for platform integration
pub struct BrowserPlatform {
    timeout_callbacks: Rc<RefCell<std::collections::HashMap<u32, u32>>>,
    next_timeout_id: Rc<RefCell<u32>>,
    animation_callbacks: Rc<RefCell<std::collections::HashMap<u32, u32>>>,
    next_animation_id: Rc<RefCell<u32>>,
}

impl Default for BrowserPlatform {
    fn default() -> Self {
        Self {
            timeout_callbacks: Rc::new(RefCell::new(std::collections::HashMap::new())),
            next_timeout_id: Rc::new(RefCell::new(1)),
            animation_callbacks: Rc::new(RefCell::new(std::collections::HashMap::new())),
            next_animation_id: Rc::new(RefCell::new(1)),
        }
    }
}

impl BrowserPlatform {
    /// Create a new browser platform instance
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the inner width of the window
    pub fn inner_width(&self) -> u32 {
        web_sys::window().unwrap().inner_width().unwrap().as_f64() as u32
    }

    /// Get the inner height of the window
    pub fn inner_height(&self) -> u32 {
        web_sys::window().unwrap().inner_height().unwrap().as_f64() as u32
    }

    /// Set a timeout callback
    pub fn set_timeout(&self, callback_id: u32, ms: i32) -> u32 {
        let id = *self.next_timeout_id.borrow();
        *self.next_timeout_id.borrow_mut() = id + 1;

        let timeout_callback_id = web_sys::window()
            .unwrap()
            .set_timeout_with_callback_and_timeout(
                &Closure::once_into_js(move || {
                    // Call the WASM callback
                    let exports = wasm_bindgen::internals::exported_function::get_or_init()
                        .lock()
                        .unwrap()
                        .get("tairitsu-browser:full/timer-callbacks@0.2.0");

                    if let Some(exports) = exports {
                        let callback = exports.as_object().unwrap().get("on_timeout");
                        if let Some(callback) = callback {
                            let callback = callback.as_function().unwrap();
                            callback.call1(&JsValue::NULL, &JsValue::from_u32(callback_id))
                                .unwrap();
                        }
                    }
                }).into_js_value(),
                ms,
            )
            .unwrap();

        self.timeout_callbacks.borrow_mut().insert(id, timeout_callback_id);
        id
    }

    /// Clear a timeout
    pub fn clear_timeout(&self, id: u32) {
        if let Some(timeout_id) = self.timeout_callbacks.borrow_mut().remove(&id) {
            web_sys::window().unwrap().clear_timeout_with_handle(timeout_id);
        }
    }

    /// Request an animation frame
    pub fn request_animation_frame(&self, callback_id: u32) -> u32 {
        let id = *self.next_animation_id.borrow();
        *self.next_animation_id.borrow_mut() = id + 1;

        let animation_callback_id = web_sys::window()
            .unwrap()
            .request_animation_frame(&Closure::once_into_js(move |timestamp: f64| {
                // Call the WASM callback
                let exports = wasm_bindgen::internals::exported_function::get_or_init()
                    .lock()
                    .unwrap()
                    .get("tairitsu-browser:full/animation-callbacks@0.2.0");

                if let Some(exports) = exports {
                    let callback = exports.as_object().unwrap().get("on_animation_frame");
                    if let Some(callback) = callback {
                        let callback = callback.as_function().unwrap();
                        callback.call2(
                            &JsValue::NULL,
                            &JsValue::from_u32(callback_id),
                            &JsValue::from_f64(timestamp),
                        )
                            .unwrap();
                    }
                }
            }).into_js_value())
            .unwrap();

        self.animation_callbacks.borrow_mut().insert(id, animation_callback_id);
        id
    }

    /// Cancel an animation frame
    pub fn cancel_animation_frame(&self, id: u32) {
        if let Some(animation_id) = self.animation_callbacks.borrow_mut().remove(&id) {
            web_sys::window().unwrap().cancel_animation_frame(animation_id);
        }
    }

    /// Get the bounding client rect of an element
    pub fn get_bounding_client_rect(&self, element_handle: u32) -> web_sys::DomRect {
        // This would need to be implemented with the element handle system
        // For now, return a default rect
        web_sys::DomRect::new().unwrap()
    }

    /// Create a resize observer
    pub fn create_resize_observer(&self, callback_id: u32) -> u32 {
        // Implementation would go here
        // For now, return a dummy handle
        1
    }

    /// Observe element resize
    pub fn observe_resize(&self, observer: u32, element: u32) {
        // Implementation would go here
    }

    /// Unobserve element resize
    pub fn unobserve_resize(&self, observer: u32, element: u32) {
        // Implementation would go here
    }

    /// Disconnect resize observer
    pub fn disconnect_resize(&self, observer: u32) {
        // Implementation would go here
    }

    /// Create a mutation observer
    pub fn create_mutation_observer(&self, callback_id: u32) -> u32 {
        // Implementation would go here
        // For now, return a dummy handle
        1
    }

    /// Observe mutations
    pub fn observe_mutations(&self, observer: u32, target: u32, options: &JsValue) {
        // Implementation would go here
    }

    /// Disconnect mutation observer
    pub fn disconnect_mutation(&self, observer: u32) {
        // Implementation would go here
    }
}

/// Global browser platform instance
pub static BROWSER_PLATFORM: std::sync::OnceLock<BrowserPlatform> = std::sync::OnceLock::new();

/// Get the global browser platform instance
pub fn get_browser_platform() -> &'static BrowserPlatform {
    BROWSER_PLATFORM.get_or_init(BrowserPlatform::new)
}