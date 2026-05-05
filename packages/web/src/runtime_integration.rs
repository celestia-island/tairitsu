//! Runtime integration for web platforms.
//!
//! This module provides functions to integrate the reactive runtime
//! with the web platform's DOM manipulation capabilities.

use std::{cell::RefCell, rc::Rc};

use tairitsu_vdom::Patch;

use crate::WitElement;

/// Initialize the reactive runtime with web platform callbacks.
///
/// This function should be called once during app initialization to set up
/// the connection between the reactive runtime and the DOM.
///
/// # Arguments
///
/// * `root_element` - The root DOM element where patches will be applied
///
/// # Example
///
/// ```no_run
/// use tairitsu_web::init_runtime;
/// use tairitsu_web::WitElement;
///
/// # fn main() -> anyhow::Result<()> {
/// let root = WitElement::from_raw(0); // Your root element handle
/// init_runtime(root);
/// # Ok(())
/// # }
/// ```
#[cfg(feature = "wit-bindings")]
pub fn init_runtime(root_element: WitElement) {
    // Store the root element for patch application
    let root_ref: Rc<RefCell<WitElement>> = Rc::new(RefCell::new(root_element));

    // Set up the schedule callback for requestAnimationFrame
    tairitsu_vdom::runtime::set_schedule_callback({
        move |callback: Box<dyn FnOnce()>| {
            // Use the WIT bindings directly for requestAnimationFrame
            #[cfg(target_family = "wasm")]
            {
                let callback_id = crate::wit_platform::wasm_impl::next_callback_id();
                crate::wit_platform::wasm_impl::ANIMATION_CALLBACKS.with(|m| {
                    m.borrow_mut().insert(
                        callback_id,
                        Some(Box::new(move |_timestamp| {
                            callback();
                        })),
                    );
                });
                crate::wit_platform::wasm_impl::bindings::tairitsu_browser::full::platform_helpers::request_animation_frame(callback_id);
            }
            #[cfg(not(target_family = "wasm"))]
            {
                let _ = callback;
                tracing::error!("request_animation_frame is only available on wasm32 targets");
            }
        }
    });

    // Set up the apply_patches callback
    tairitsu_vdom::runtime::set_apply_patches_callback({
        let root_ref_clone: Rc<RefCell<WitElement>> = Rc::clone(&root_ref);
        move |_component_id: tairitsu_vdom::ComponentId, patches: Vec<Patch>| {
            let root = root_ref_clone.borrow();
            #[cfg(target_family = "wasm")]
            {
                if let Ok(platform) = crate::WitPlatform::new() {
                    if let Err(e) = platform.apply_patches(&root, &patches) {
                        tracing::error!("Failed to apply patches: {:?}", e);
                    }
                } else {
                    tracing::error!("Failed to create platform for patch application");
                }
            }
            #[cfg(not(target_family = "wasm"))]
            {
                let _ = (root, patches);
                tracing::error!("apply_patches is only available on wasm32 targets");
            }
        }
    });

    tracing::info!("Runtime initialized with web platform callbacks");
}

/// A component renderer that integrates with the reactive runtime.
///
/// This struct provides a way to render components that automatically
/// update when their signals change.
#[cfg(feature = "wit-bindings")]
pub struct ComponentRenderer {
    root_element: Rc<RefCell<WitElement>>,
}

#[cfg(feature = "wit-bindings")]
impl ComponentRenderer {
    /// Create a new component renderer.
    ///
    /// # Arguments
    ///
    /// * `root_element` - The root DOM element for rendering
    pub fn new(root_element: WitElement) -> Self {
        Self {
            root_element: Rc::new(RefCell::new(root_element)),
        }
    }

    /// Initialize the runtime with this renderer's root element.
    pub fn init_runtime(&self) {
        let root = *self.root_element.borrow();
        init_runtime(root);
    }

    /// Mount a VNode to the root element.
    pub fn mount(&self, vnode: tairitsu_vdom::VNode) -> anyhow::Result<()> {
        #[cfg(target_family = "wasm")]
        {
            if let Ok(platform) = crate::WitPlatform::new() {
                platform.mount_vnode_to_app(vnode)
            } else {
                anyhow::bail!("Failed to create platform for mounting")
            }
        }
        #[cfg(not(target_family = "wasm"))]
        {
            let _ = vnode;
            anyhow::bail!("mount is only available on wasm32 targets")
        }
    }
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_init_runtime() {
        // This test verifies that the runtime can be initialized
        // In a real environment, this would require an actual WASM context
        // For now, we just test that the function compiles
    }
}
